package com.elephantnote.androidvault

import android.app.Activity
import android.content.Intent
import android.net.Uri
import androidx.activity.result.ActivityResult
import androidx.documentfile.provider.DocumentFile
import app.tauri.Logger
import app.tauri.annotation.ActivityCallback
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import java.io.File
import java.net.URLConnection

@InvokeArg
class ShadowArgs {
  var shadowPath: String = ""
}

@InvokeArg
class ShareTextArgs {
  var title: String = "Note"
  var text: String = ""
}

@TauriPlugin
class ElephantAndroidVaultPlugin(private val activity: Activity) : Plugin(activity) {
  private val preferences by lazy {
    activity.getSharedPreferences("elephant_android_vault", Activity.MODE_PRIVATE)
  }
  private var pendingShadowPath: String? = null

  @Command
  fun pickTree(invoke: Invoke) {
    try {
      val args = invoke.parseArgs(ShadowArgs::class.java)
      require(args.shadowPath.isNotBlank()) { "A private shadow path is required." }
      pendingShadowPath = args.shadowPath
      val intent = Intent(Intent.ACTION_OPEN_DOCUMENT_TREE).apply {
        addFlags(
          Intent.FLAG_GRANT_READ_URI_PERMISSION or
            Intent.FLAG_GRANT_WRITE_URI_PERMISSION or
            Intent.FLAG_GRANT_PERSISTABLE_URI_PERMISSION or
            Intent.FLAG_GRANT_PREFIX_URI_PERMISSION
        )
      }
      startActivityForResult(invoke, intent, "treePickerResult")
    } catch (error: Exception) {
      invoke.reject(error.message ?: "Unable to open Android folder picker.")
    }
  }

  @ActivityCallback
  fun treePickerResult(invoke: Invoke, result: ActivityResult) {
    val shadowPath = pendingShadowPath
    pendingShadowPath = null
    if (result.resultCode == Activity.RESULT_CANCELED) {
      invoke.reject("Folder picker cancelled")
      return
    }
    if (result.resultCode != Activity.RESULT_OK || shadowPath.isNullOrBlank()) {
      invoke.reject("Android did not return a usable folder.")
      return
    }
    val uri = result.data?.data
    if (uri == null) {
      invoke.reject("Android did not return a document-tree URI.")
      return
    }
    try {
      val requestedFlags = result.data?.flags ?: 0
      val persistableFlags = requestedFlags and
        (Intent.FLAG_GRANT_READ_URI_PERMISSION or Intent.FLAG_GRANT_WRITE_URI_PERMISSION)
      activity.contentResolver.takePersistableUriPermission(uri, persistableFlags)
      val root = DocumentFile.fromTreeUri(activity, uri)
        ?: throw IllegalStateException("The selected folder cannot be opened.")
      if (!root.canRead() || !root.canWrite()) {
        throw IllegalStateException("The selected folder is not readable and writable.")
      }
      preferences.edit()
        .putString("tree_uri", uri.toString())
        .putString("display_name", root.name ?: "Android vault")
        .apply()
      val copied = restoreTree(root, File(shadowPath))
      invoke.resolve(state(shadowPath, uri, root.name, copied))
    } catch (error: Exception) {
      Logger.error(error.message ?: "Unable to persist Android folder access.")
      invoke.reject(error.message ?: "Unable to persist Android folder access.")
    }
  }

  @Command
  fun restore(invoke: Invoke) {
    val args = invoke.parseArgs(ShadowArgs::class.java)
    try {
      val uri = persistedUri()
      if (uri == null) {
        invoke.resolve(state(args.shadowPath, null, null, 0))
        return
      }
      val root = openRoot(uri)
      val copied = restoreTree(root, File(args.shadowPath))
      invoke.resolve(state(args.shadowPath, uri, root.name, copied))
    } catch (error: Exception) {
      invoke.reject(error.message ?: "Unable to restore the Android vault.")
    }
  }

  @Command
  fun syncToTree(invoke: Invoke) {
    val args = invoke.parseArgs(ShadowArgs::class.java)
    try {
      val uri = persistedUri()
      if (uri == null) {
        invoke.resolve(state(args.shadowPath, null, null, 0))
        return
      }
      val root = openRoot(uri)
      val copied = syncDirectory(File(args.shadowPath), root)
      invoke.resolve(state(args.shadowPath, uri, root.name, copied))
    } catch (error: Exception) {
      invoke.reject(error.message ?: "Unable to synchronize the Android vault.")
    }
  }

  @Command
  fun shareText(invoke: Invoke) {
    try {
      val args = invoke.parseArgs(ShareTextArgs::class.java)
      val sendIntent = Intent(Intent.ACTION_SEND).apply {
        type = "text/plain"
        putExtra(Intent.EXTRA_TITLE, args.title)
        putExtra(Intent.EXTRA_SUBJECT, args.title)
        putExtra(Intent.EXTRA_TEXT, args.text)
      }
      activity.startActivity(Intent.createChooser(sendIntent, args.title.ifBlank { "Share note" }))
      invoke.resolve()
    } catch (error: Exception) {
      invoke.reject(error.message ?: "Unable to open Android sharing.")
    }
  }

  @Command
  fun clear(invoke: Invoke) {
    val args = invoke.parseArgs(ShadowArgs::class.java)
    val oldUri = persistedUri()
    if (oldUri != null) {
      try {
        activity.contentResolver.releasePersistableUriPermission(
          oldUri,
          Intent.FLAG_GRANT_READ_URI_PERMISSION or Intent.FLAG_GRANT_WRITE_URI_PERMISSION
        )
      } catch (_: SecurityException) {
      }
    }
    preferences.edit().clear().apply()
    invoke.resolve(state(args.shadowPath, null, null, 0))
  }

  private fun persistedUri(): Uri? {
    val uri = preferences.getString("tree_uri", null)?.let(Uri::parse) ?: return null
    val granted = activity.contentResolver.persistedUriPermissions.any { permission ->
      permission.uri == uri && permission.isReadPermission && permission.isWritePermission
    }
    if (!granted) {
      preferences.edit().clear().apply()
      return null
    }
    return uri
  }

  private fun openRoot(uri: Uri): DocumentFile {
    val root = DocumentFile.fromTreeUri(activity, uri)
      ?: throw IllegalStateException("The saved Android folder is no longer available.")
    if (!root.canRead() || !root.canWrite()) {
      throw SecurityException("Android revoked access to the selected vault folder.")
    }
    return root
  }

  private fun restoreTree(root: DocumentFile, shadow: File): Long {
    if (shadow.exists()) shadow.listFiles()?.forEach { it.deleteRecursively() }
    shadow.mkdirs()
    return copyDocumentDirectory(root, shadow)
  }

  private fun copyDocumentDirectory(source: DocumentFile, destination: File): Long {
    var copied = 0L
    for (child in source.listFiles()) {
      val name = child.name ?: continue
      val target = File(destination, name)
      if (child.isDirectory) {
        target.mkdirs()
        copied += copyDocumentDirectory(child, target)
      } else if (child.isFile) {
        target.parentFile?.mkdirs()
        activity.contentResolver.openInputStream(child.uri).use { input ->
          requireNotNull(input) { "Unable to read $name from the selected vault." }
          target.outputStream().use { output -> input.copyTo(output) }
        }
        copied += 1
      }
    }
    return copied
  }

  private fun syncDirectory(local: File, remote: DocumentFile): Long {
    local.mkdirs()
    val localChildren = local.listFiles()?.associateBy { it.name } ?: emptyMap()
    for (remoteChild in remote.listFiles()) {
      val name = remoteChild.name ?: continue
      if (!localChildren.containsKey(name)) remoteChild.delete()
    }
    var copied = 0L
    for ((name, child) in localChildren) {
      if (child.isDirectory) {
        val target = remote.findFile(name)?.takeIf { it.isDirectory }
          ?: remote.createDirectory(name)
          ?: throw IllegalStateException("Unable to create folder $name.")
        copied += syncDirectory(child, target)
      } else if (child.isFile) {
        var target = remote.findFile(name)
        if (target == null || !target.isFile) {
          target?.delete()
          val mime = URLConnection.guessContentTypeFromName(name) ?: "application/octet-stream"
          target = remote.createFile(mime, name)
        }
        target ?: throw IllegalStateException("Unable to create file $name.")
        activity.contentResolver.openOutputStream(target.uri, "wt").use { output ->
          requireNotNull(output) { "Unable to write $name to the selected vault." }
          child.inputStream().use { input -> input.copyTo(output) }
        }
        copied += 1
      }
    }
    return copied
  }

  private fun state(
    shadowPath: String,
    uri: Uri?,
    displayName: String?,
    filesCopied: Long
  ): JSObject = JSObject().apply {
    put("configured", uri != null)
    put("uri", uri?.toString())
    put("displayName", displayName)
    put("shadowPath", shadowPath)
    put("filesCopied", filesCopied)
  }
}
