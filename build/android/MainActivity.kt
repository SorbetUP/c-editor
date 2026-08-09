package com.elephantnote.app

import android.os.Bundle
import android.view.View
import android.view.WindowInsets
import android.view.WindowInsetsController

class MainActivity : TauriActivity() {
  override fun onCreate(savedInstanceState: Bundle?) {
    super.onCreate(savedInstanceState)
    // Camera access is requested only after an explicit user action.
    scheduleImmersiveMode()
  }

  override fun onResume() {
    super.onResume()
    scheduleImmersiveMode()
  }

  override fun onWindowFocusChanged(hasFocus: Boolean) {
    super.onWindowFocusChanged(hasFocus)
    if (hasFocus) scheduleImmersiveMode()
  }

  private fun scheduleImmersiveMode() {
    val decorView = window.decorView
    decorView.post { enterImmersiveMode(decorView) }
  }

  private fun enterImmersiveMode(decorView: View) {
    if (android.os.Build.VERSION.SDK_INT >= android.os.Build.VERSION_CODES.R) {
      window.setDecorFitsSystemWindows(false)
      decorView.windowInsetsController?.let { controller ->
        controller.hide(WindowInsets.Type.systemBars())
        controller.systemBarsBehavior =
          WindowInsetsController.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE
      }
    } else {
      @Suppress("DEPRECATION")
      decorView.systemUiVisibility =
        View.SYSTEM_UI_FLAG_IMMERSIVE_STICKY or
          View.SYSTEM_UI_FLAG_FULLSCREEN or
          View.SYSTEM_UI_FLAG_HIDE_NAVIGATION or
          View.SYSTEM_UI_FLAG_LAYOUT_FULLSCREEN or
          View.SYSTEM_UI_FLAG_LAYOUT_HIDE_NAVIGATION or
          View.SYSTEM_UI_FLAG_LAYOUT_STABLE
    }
  }
}
