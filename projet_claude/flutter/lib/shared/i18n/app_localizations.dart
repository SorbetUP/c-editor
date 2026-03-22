import 'package:flutter/material.dart';

class AppLocalizations {
  final Locale locale;

  AppLocalizations(this.locale);

  static AppLocalizations of(BuildContext context) {
    return Localizations.of<AppLocalizations>(context, AppLocalizations)!;
  }

  static const LocalizationsDelegate<AppLocalizations> delegate =
      _AppLocalizationsDelegate();

  static const List<Locale> supportedLocales = [
    Locale('fr', ''),
    Locale('en', ''),
  ];

  // App general
  String get appTitle => locale.languageCode == 'fr' ? 'Éditeur de Notes' : 'Note Editor';
  String get loading => locale.languageCode == 'fr' ? 'Chargement...' : 'Loading...';
  String get error => locale.languageCode == 'fr' ? 'Erreur' : 'Error';
  String get cancel => locale.languageCode == 'fr' ? 'Annuler' : 'Cancel';
  String get save => locale.languageCode == 'fr' ? 'Enregistrer' : 'Save';
  String get delete => locale.languageCode == 'fr' ? 'Supprimer' : 'Delete';
  String get edit => locale.languageCode == 'fr' ? 'Modifier' : 'Edit';
  String get create => locale.languageCode == 'fr' ? 'Créer' : 'Create';
  String get back => locale.languageCode == 'fr' ? 'Retour' : 'Back';

  // Home screen
  String get myNotes => locale.languageCode == 'fr' ? 'Mes Notes' : 'My Notes';
  String get noNotes => locale.languageCode == 'fr' ? 'Aucune note' : 'No notes';
  String get createFirstNote => locale.languageCode == 'fr' 
      ? 'Créez votre première note en appuyant sur le bouton +' 
      : 'Create your first note by tapping the + button';
  String get createNote => locale.languageCode == 'fr' ? 'Créer une note' : 'Create note';
  String get newNote => locale.languageCode == 'fr' ? 'Nouvelle note' : 'New note';
  String get noteName => locale.languageCode == 'fr' ? 'Nom de la note' : 'Note name';
  String get myNewNote => locale.languageCode == 'fr' ? 'Ma nouvelle note' : 'My new note';
  String get enterName => locale.languageCode == 'fr' ? 'Veuillez entrer un nom' : 'Please enter a name';

  // Import/Export
  String get import => locale.languageCode == 'fr' ? 'Importer' : 'Import';
  String get export => locale.languageCode == 'fr' ? 'Exporter' : 'Export';
  String get importMarkdown => locale.languageCode == 'fr' ? 'Importer Markdown' : 'Import Markdown';
  String get convertMdFile => locale.languageCode == 'fr' 
      ? 'Convertir un fichier .md en note' 
      : 'Convert a .md file to note';
  String get openJson => locale.languageCode == 'fr' ? 'Ouvrir JSON' : 'Open JSON';
  String get openJsonFile => locale.languageCode == 'fr' 
      ? 'Ouvrir un fichier note.json' 
      : 'Open a note.json file';

  // Folder operations
  String get folder => locale.languageCode == 'fr' ? 'Dossier' : 'Folder';
  String get changeFolder => locale.languageCode == 'fr' ? 'Changer de dossier' : 'Change folder';
  String get selectNotesFolder => locale.languageCode == 'fr' 
      ? 'Sélectionner un autre dossier de notes' 
      : 'Select another notes folder';
  String get refresh => locale.languageCode == 'fr' ? 'Actualiser' : 'Refresh';
  String get reloadNotesList => locale.languageCode == 'fr' 
      ? 'Recharger la liste des notes' 
      : 'Reload notes list';

  // Viewer screen
  String get viewer => locale.languageCode == 'fr' ? 'Visualiseur' : 'Viewer';
  String get noNoteSelected => locale.languageCode == 'fr' 
      ? 'Aucune note sélectionnée' 
      : 'No note selected';
  String get loadingError => locale.languageCode == 'fr' 
      ? 'Erreur de chargement' 
      : 'Loading error';
  String get noteNotFound => locale.languageCode == 'fr' ? 'Note introuvable' : 'Note not found';
  String get noteDeletedOrMissing => locale.languageCode == 'fr' 
      ? 'Cette note n\'existe pas ou a été supprimée.' 
      : 'This note does not exist or has been deleted.';
  String get backToHome => locale.languageCode == 'fr' ? 'Retour à l\'accueil' : 'Back to home';
  String get reloadFromDisk => locale.languageCode == 'fr' 
      ? 'Recharger depuis le disque' 
      : 'Reload from disk';
  String get exportMarkdown => locale.languageCode == 'fr' 
      ? 'Exporter en Markdown' 
      : 'Export to Markdown';
  String get documentInfo => locale.languageCode == 'fr' 
      ? 'Informations du document' 
      : 'Document information';

  // Settings screen
  String get settings => locale.languageCode == 'fr' ? 'Paramètres' : 'Settings';
  String get fontTypography => locale.languageCode == 'fr' 
      ? 'Police et typographie' 
      : 'Font and typography';
  String get theme => locale.languageCode == 'fr' ? 'Thème' : 'Theme';
  String get language => locale.languageCode == 'fr' ? 'Langue' : 'Language';
  String get storage => locale.languageCode == 'fr' ? 'Stockage' : 'Storage';
  String get performance => locale.languageCode == 'fr' ? 'Performance' : 'Performance';
  String get about => locale.languageCode == 'fr' ? 'À propos' : 'About';

  // Time formatting
  String get justNow => locale.languageCode == 'fr' ? 'à l\'instant' : 'just now';
  String get yesterday => locale.languageCode == 'fr' ? 'hier' : 'yesterday';
  String minutesAgo(int minutes) => locale.languageCode == 'fr' 
      ? 'il y a $minutes min' 
      : '$minutes min ago';
  String hoursAgo(int hours) => locale.languageCode == 'fr' 
      ? 'il y a $hours h' 
      : '$hours h ago';
  String daysAgo(int days) => locale.languageCode == 'fr' 
      ? 'il y a $days jours' 
      : '$days days ago';

  // Messages
  String get noteCreated => locale.languageCode == 'fr' ? 'Note créée' : 'Note created';
  String get noteDeleted => locale.languageCode == 'fr' ? 'Note supprimée' : 'Note deleted';
  String get noteSaved => locale.languageCode == 'fr' ? 'Note sauvegardée' : 'Note saved';
  String get documentReloaded => locale.languageCode == 'fr' ? 'Document rechargé' : 'Document reloaded';
  String markdownExported(String filename) => locale.languageCode == 'fr' 
      ? 'Markdown exporté: $filename' 
      : 'Markdown exported: $filename';
  String folderChanged(String path) => locale.languageCode == 'fr' 
      ? 'Dossier changé: $path' 
      : 'Folder changed: $path';
  String get listRefreshed => locale.languageCode == 'fr' ? 'Liste actualisée' : 'List refreshed';
}

class _AppLocalizationsDelegate extends LocalizationsDelegate<AppLocalizations> {
  const _AppLocalizationsDelegate();

  @override
  bool isSupported(Locale locale) {
    return AppLocalizations.supportedLocales
        .any((supportedLocale) => supportedLocale.languageCode == locale.languageCode);
  }

  @override
  Future<AppLocalizations> load(Locale locale) async {
    return AppLocalizations(locale);
  }

  @override
  bool shouldReload(_AppLocalizationsDelegate old) => false;
}