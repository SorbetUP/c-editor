// VaultSetupController.m - Implémentation de l'interface de configuration du vault
#import "VaultSetupController.h"

@implementation VaultSetupController

- (instancetype)init {
    self = [super initWithWindowNibName:nil];
    if (self) {
        _setupComplete = NO;
        _selectedVaultPath = nil;
        
        // Initialiser le gestionnaire de vaults
        vault_manager_init();
    }
    return self;
}

- (void)loadWindow {
    // Créer la fenêtre programmatiquement
    NSRect windowFrame = NSMakeRect(0, 0, 600, 500);
    NSWindow* window = [[NSWindow alloc] 
        initWithContentRect:windowFrame
        styleMask:(NSWindowStyleMaskTitled | NSWindowStyleMaskClosable)
        backing:NSBackingStoreBuffered 
        defer:NO];
    
    [window setTitle:@"Configuration d'ElephantNotes"];
    [window center];
    [window setLevel:NSFloatingWindowLevel];
    [window setMovable:YES];
    
    self.window = window;
    [self setupUI];
}

- (void)setupUI {
    NSView* contentView = [self.window contentView];
    
    // Titre principal
    NSTextField* titleLabel = [[NSTextField alloc] initWithFrame:NSMakeRect(50, 420, 500, 40)];
    [titleLabel setStringValue:@"🐘 Bienvenue dans ElephantNotes Professional"];
    [titleLabel setFont:[NSFont systemFontOfSize:24 weight:NSFontWeightBold]];
    [titleLabel setTextColor:[NSColor labelColor]];
    [titleLabel setBackgroundColor:[NSColor clearColor]];
    [titleLabel setBordered:NO];
    [titleLabel setEditable:NO];
    [titleLabel setAlignment:NSTextAlignmentCenter];
    [contentView addSubview:titleLabel];
    
    // Texte d'explication
    _welcomeLabel = [[NSTextField alloc] initWithFrame:NSMakeRect(50, 360, 500, 50)];
    [_welcomeLabel setStringValue:@"Pour commencer, créons votre premier vault.\nUn vault est un dossier qui contiendra toutes vos notes et documents."];
    [_welcomeLabel setFont:[NSFont systemFontOfSize:14]];
    [_welcomeLabel setTextColor:[NSColor secondaryLabelColor]];
    [_welcomeLabel setBackgroundColor:[NSColor clearColor]];
    [_welcomeLabel setBordered:NO];
    [_welcomeLabel setEditable:NO];
    [_welcomeLabel setAlignment:NSTextAlignmentCenter];
    [contentView addSubview:_welcomeLabel];
    
    // Nom du vault
    NSTextField* nameLabel = [[NSTextField alloc] initWithFrame:NSMakeRect(50, 320, 100, 20)];
    [nameLabel setStringValue:@"Nom du vault :"];
    [nameLabel setFont:[NSFont systemFontOfSize:13 weight:NSFontWeightMedium]];
    [nameLabel setBackgroundColor:[NSColor clearColor]];
    [nameLabel setBordered:NO];
    [nameLabel setEditable:NO];
    [contentView addSubview:nameLabel];
    
    _vaultNameField = [[NSTextField alloc] initWithFrame:NSMakeRect(50, 295, 500, 25)];
    [_vaultNameField setStringValue:@"Mon Vault"];
    [_vaultNameField setFont:[NSFont systemFontOfSize:13]];
    [_vaultNameField setTarget:self];
    [_vaultNameField setAction:@selector(vaultNameChanged:)];
    [contentView addSubview:_vaultNameField];
    
    // Emplacement du vault
    NSTextField* locationLabel = [[NSTextField alloc] initWithFrame:NSMakeRect(50, 260, 150, 20)];
    [locationLabel setStringValue:@"Emplacement :"];
    [locationLabel setFont:[NSFont systemFontOfSize:13 weight:NSFontWeightMedium]];
    [locationLabel setBackgroundColor:[NSColor clearColor]];
    [locationLabel setBordered:NO];
    [locationLabel setEditable:NO];
    [contentView addSubview:locationLabel];
    
    _vaultLocationField = [[NSTextField alloc] initWithFrame:NSMakeRect(50, 235, 400, 25)];
    [_vaultLocationField setEditable:NO];
    [_vaultLocationField setFont:[NSFont systemFontOfSize:13]];
    [contentView addSubview:_vaultLocationField];
    
    _browseButton = [[NSButton alloc] initWithFrame:NSMakeRect(460, 235, 90, 25)];
    [_browseButton setTitle:@"Parcourir..."];
    [_browseButton setTarget:self];
    [_browseButton setAction:@selector(browseForLocation:)];
    [contentView addSubview:_browseButton];
    
    // Description
    NSTextField* descLabel = [[NSTextField alloc] initWithFrame:NSMakeRect(50, 200, 150, 20)];
    [descLabel setStringValue:@"Description (optionnel) :"];
    [descLabel setFont:[NSFont systemFontOfSize:13 weight:NSFontWeightMedium]];
    [descLabel setBackgroundColor:[NSColor clearColor]];
    [descLabel setBordered:NO];
    [descLabel setEditable:NO];
    [contentView addSubview:descLabel];
    
    NSScrollView* descScrollView = [[NSScrollView alloc] initWithFrame:NSMakeRect(50, 140, 500, 60)];
    [descScrollView setHasVerticalScroller:YES];
    [descScrollView setAutohidesScrollers:YES];
    _descriptionTextView = [[NSTextView alloc] init];
    [_descriptionTextView setString:@"Mon premier vault ElephantNotes"];
    [_descriptionTextView setFont:[NSFont systemFontOfSize:13]];
    [descScrollView setDocumentView:_descriptionTextView];
    [contentView addSubview:descScrollView];
    
    // Créer des notes d'exemple
    _createSamplesCheckbox = [[NSButton alloc] initWithFrame:NSMakeRect(50, 110, 300, 20)];
    [_createSamplesCheckbox setButtonType:NSButtonTypeSwitch];
    [_createSamplesCheckbox setTitle:@"Créer des notes d'exemple pour démarrer"];
    [_createSamplesCheckbox setState:NSControlStateValueOn];
    [_createSamplesCheckbox setFont:[NSFont systemFontOfSize:13]];
    [contentView addSubview:_createSamplesCheckbox];
    
    // Boutons
    _cancelButton = [[NSButton alloc] initWithFrame:NSMakeRect(370, 30, 80, 32)];
    [_cancelButton setTitle:@"Annuler"];
    [_cancelButton setTarget:self];
    [_cancelButton setAction:@selector(cancelSetup:)];
    [contentView addSubview:_cancelButton];
    
    _createButton = [[NSButton alloc] initWithFrame:NSMakeRect(460, 30, 90, 32)];
    [_createButton setTitle:@"Créer"];
    [_createButton setTarget:self];
    [_createButton setAction:@selector(createVault:)];
    [_createButton setKeyEquivalent:@"\r"];
    [contentView addSubview:_createButton];
    
    // Indicateur de progression
    _progressIndicator = [[NSProgressIndicator alloc] initWithFrame:NSMakeRect(50, 35, 200, 20)];
    [_progressIndicator setStyle:NSProgressIndicatorStyleBar];
    [_progressIndicator setIndeterminate:YES];
    [_progressIndicator setHidden:YES];
    [contentView addSubview:_progressIndicator];
    
    // Label de statut
    _statusLabel = [[NSTextField alloc] initWithFrame:NSMakeRect(50, 10, 300, 20)];
    [_statusLabel setStringValue:@""];
    [_statusLabel setFont:[NSFont systemFontOfSize:12]];
    [_statusLabel setTextColor:[NSColor secondaryLabelColor]];
    [_statusLabel setBackgroundColor:[NSColor clearColor]];
    [_statusLabel setBordered:NO];
    [_statusLabel setEditable:NO];
    [_statusLabel setHidden:YES];
    [contentView addSubview:_statusLabel];
    
    // Initialiser l'emplacement par défaut
    [self updateDefaultLocation];
}

- (void)updateDefaultLocation {
    char* default_location = NULL;
    VaultResult result = vault_get_default_vault_location(&default_location);
    
    if (result == VAULT_SUCCESS && default_location) {
        NSString* defaultPath = [NSString stringWithUTF8String:default_location];
        [_vaultLocationField setStringValue:defaultPath];
        vault_free_string(default_location);
    } else {
        // Fallback vers Documents
        NSString* documentsPath = [NSHomeDirectory() stringByAppendingPathComponent:@"Documents"];
        [_vaultLocationField setStringValue:documentsPath];
    }
}

- (void)showSetupWindow {
    [self loadWindow];
    [self.window makeKeyAndOrderFront:nil];
    [NSApp runModalForWindow:self.window];
}

- (IBAction)browseForLocation:(id)sender {
    NSOpenPanel* openPanel = [NSOpenPanel openPanel];
    [openPanel setCanChooseFiles:NO];
    [openPanel setCanChooseDirectories:YES];
    [openPanel setCanCreateDirectories:YES];
    [openPanel setAllowsMultipleSelection:NO];
    [openPanel setTitle:@"Choisir l'emplacement du vault"];
    [openPanel setPrompt:@"Sélectionner"];
    
    NSString* currentLocation = [_vaultLocationField stringValue];
    if (currentLocation && [currentLocation length] > 0) {
        [openPanel setDirectoryURL:[NSURL fileURLWithPath:currentLocation]];
    }
    
    [openPanel beginSheetModalForWindow:self.window completionHandler:^(NSInteger result) {
        if (result == NSModalResponseOK) {
            NSURL* selectedURL = [[openPanel URLs] firstObject];
            NSString* selectedPath = [selectedURL path];
            [self->_vaultLocationField setStringValue:selectedPath];
        }
    }];
}

- (IBAction)vaultNameChanged:(id)sender {
    // Valider et mettre à jour le nom si nécessaire
    NSString* name = [_vaultNameField stringValue];
    if ([name length] == 0) {
        [_vaultNameField setStringValue:@"Mon Vault"];
    }
}

- (BOOL)validateInput {
    NSString* name = [_vaultNameField stringValue];
    NSString* location = [_vaultLocationField stringValue];
    
    if (!name || [name length] == 0) {
        [self showError:@"Veuillez entrer un nom pour le vault."];
        return NO;
    }
    
    if (!location || [location length] == 0) {
        [self showError:@"Veuillez sélectionner un emplacement pour le vault."];
        return NO;
    }
    
    // Vérifier que l'emplacement est accessible
    NSFileManager* fileManager = [NSFileManager defaultManager];
    if (![fileManager fileExistsAtPath:location]) {
        [self showError:@"L'emplacement sélectionné n'existe pas."];
        return NO;
    }
    
    // Vérifier les permissions d'écriture
    if (![fileManager isWritableFileAtPath:location]) {
        [self showError:@"Vous n'avez pas les permissions d'écriture pour cet emplacement."];
        return NO;
    }
    
    // Vérifier que le vault n'existe pas déjà
    NSString* fullPath = [location stringByAppendingPathComponent:name];
    if ([fileManager fileExistsAtPath:fullPath]) {
        [self showError:@"Un dossier avec ce nom existe déjà à cet emplacement."];
        return NO;
    }
    
    return YES;
}

- (IBAction)createVault:(id)sender {
    if (![self validateInput]) return;
    
    // Afficher la progression
    [_progressIndicator setHidden:NO];
    [_progressIndicator startAnimation:nil];
    [_statusLabel setStringValue:@"Création du vault..."];
    [_statusLabel setHidden:NO];
    [_createButton setEnabled:NO];
    [_cancelButton setEnabled:NO];
    
    dispatch_async(dispatch_get_global_queue(DISPATCH_QUEUE_PRIORITY_DEFAULT, 0), ^{
        VaultCreationOptions options = {0};
        
        NSString* name = [self->_vaultNameField stringValue];
        NSString* location = [self->_vaultLocationField stringValue];
        NSString* description = [self->_descriptionTextView string];
        BOOL createSamples = [self->_createSamplesCheckbox state] == NSControlStateValueOn;
        
        options.name = (char*)[name UTF8String];
        options.path = (char*)[location UTF8String];
        options.description = (char*)[description UTF8String];
        options.type = VAULT_TYPE_LOCAL;
        options.encrypt = false;
        options.password = NULL;
        options.create_sample_notes = createSamples;
        options.template_name = NULL;
        
        VaultInfo* info = NULL;
        VaultResult result = vault_create(&options, &info);
        
        dispatch_async(dispatch_get_main_queue(), ^{
            [self->_progressIndicator stopAnimation:nil];
            [self->_progressIndicator setHidden:YES];
            [self->_statusLabel setHidden:YES];
            [self->_createButton setEnabled:YES];
            [self->_cancelButton setEnabled:YES];
            
            if (result == VAULT_SUCCESS && info) {
                // Ajouter au registre
                VaultRegistry* registry = NULL;
                vault_registry_load(&registry);
                if (!registry) {
                    registry = malloc(sizeof(VaultRegistry));
                    memset(registry, 0, sizeof(VaultRegistry));
                }
                
                NSString* vaultPath = [location stringByAppendingPathComponent:name];
                vault_registry_add(registry, [vaultPath UTF8String]);
                vault_registry_set_default(registry, [vaultPath UTF8String]);
                vault_registry_save(registry);
                vault_registry_free(registry);
                
                // Marquer la configuration comme terminée
                vault_mark_first_launch_complete();
                
                self->_selectedVaultPath = vaultPath;
                self->_setupComplete = YES;
                
                [self showSuccess:@"Vault créé avec succès !"];
                
                // Fermer après 1 seconde
                dispatch_after(dispatch_time(DISPATCH_TIME_NOW, 1 * NSEC_PER_SEC), dispatch_get_main_queue(), ^{
                    [self closeSetup:YES];
                });
                
                vault_free_info(info);
                free(info);
            } else {
                const char* error_msg = vault_get_error_message(result);
                NSString* errorString = [NSString stringWithUTF8String:error_msg];
                [self showError:[NSString stringWithFormat:@"Erreur lors de la création : %@", errorString]];
            }
        });
    });
}

- (IBAction)cancelSetup:(id)sender {
    [self closeSetup:NO];
}

- (void)closeSetup:(BOOL)success {
    [NSApp stopModal];
    [self.window orderOut:nil];
    
    if (self.completionHandler) {
        self.completionHandler(success, _selectedVaultPath);
    }
}

- (void)showError:(NSString*)message {
    [_statusLabel setStringValue:message];
    [_statusLabel setTextColor:[NSColor systemRedColor]];
    [_statusLabel setHidden:NO];
    
    // Masquer après 5 secondes
    dispatch_after(dispatch_time(DISPATCH_TIME_NOW, 5 * NSEC_PER_SEC), dispatch_get_main_queue(), ^{
        [self->_statusLabel setHidden:YES];
        [self->_statusLabel setTextColor:[NSColor secondaryLabelColor]];
    });
}

- (void)showSuccess:(NSString*)message {
    [_statusLabel setStringValue:message];
    [_statusLabel setTextColor:[NSColor systemGreenColor]];
    [_statusLabel setHidden:NO];
}

- (void)updateSuggestedName {
    // This method can be implemented later if needed
    // For now, we use a fixed default name
}

- (void)dealloc {
    vault_manager_cleanup();
}

@end