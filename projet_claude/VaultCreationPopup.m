// VaultCreationPopup.m - Implémentation de la fenêtre popup de création de vault
#import "VaultCreationPopup.h"

@implementation VaultCreationPopup

@synthesize delegate;

- (instancetype)init {
    // Créer la fenêtre popup
    NSRect windowFrame = NSMakeRect(0, 0, 500, 400);
    NSWindow* window = [[NSWindow alloc] 
        initWithContentRect:windowFrame
        styleMask:(NSWindowStyleMaskTitled | NSWindowStyleMaskClosable)
        backing:NSBackingStoreBuffered 
        defer:NO];
    
    self = [super initWithWindow:window];
    if (self) {
        [window setTitle:@"Créer un Nouveau Vault"];
        [window center];
        [window setLevel:NSFloatingWindowLevel];
        [window setMovable:YES];
        
        [self setupUI];
    }
    return self;
}

- (void)setupUI {
    NSView* contentView = [self.window contentView];
    
    // Titre
    NSTextField* titleLabel = [[NSTextField alloc] initWithFrame:NSMakeRect(20, 350, 460, 30)];
    [titleLabel setStringValue:@"Bienvenue dans ElephantNotes V3"];
    [titleLabel setFont:[NSFont systemFontOfSize:18 weight:NSFontWeightBold]];
    [titleLabel setTextColor:[NSColor labelColor]];
    [titleLabel setBackgroundColor:[NSColor clearColor]];
    [titleLabel setBordered:NO];
    [titleLabel setEditable:NO];
    [titleLabel setAlignment:NSTextAlignmentCenter];
    [contentView addSubview:titleLabel];
    
    // Description
    NSTextField* descLabel = [[NSTextField alloc] initWithFrame:NSMakeRect(20, 310, 460, 40)];
    [descLabel setStringValue:@"Pour commencer, créons votre premier vault.\nUn vault contiendra toutes vos notes et documents."];
    [descLabel setFont:[NSFont systemFontOfSize:13]];
    [descLabel setTextColor:[NSColor secondaryLabelColor]];
    [descLabel setBackgroundColor:[NSColor clearColor]];
    [descLabel setBordered:NO];
    [descLabel setEditable:NO];
    [descLabel setAlignment:NSTextAlignmentCenter];
    [contentView addSubview:descLabel];
    
    // Nom du vault
    NSTextField* nameLabel = [[NSTextField alloc] initWithFrame:NSMakeRect(20, 270, 100, 20)];
    [nameLabel setStringValue:@"Nom du vault :"];
    [nameLabel setFont:[NSFont systemFontOfSize:13 weight:NSFontWeightMedium]];
    [nameLabel setBackgroundColor:[NSColor clearColor]];
    [nameLabel setBordered:NO];
    [nameLabel setEditable:NO];
    [contentView addSubview:nameLabel];
    
    _nameField = [[NSTextField alloc] initWithFrame:NSMakeRect(20, 245, 460, 25)];
    [_nameField setStringValue:@"Mon Premier Vault"];
    [_nameField setFont:[NSFont systemFontOfSize:13]];
    [contentView addSubview:_nameField];
    
    // Emplacement
    NSTextField* locationLabel = [[NSTextField alloc] initWithFrame:NSMakeRect(20, 210, 150, 20)];
    [locationLabel setStringValue:@"Emplacement :"];
    [locationLabel setFont:[NSFont systemFontOfSize:13 weight:NSFontWeightMedium]];
    [locationLabel setBackgroundColor:[NSColor clearColor]];
    [locationLabel setBordered:NO];
    [locationLabel setEditable:NO];
    [contentView addSubview:locationLabel];
    
    _locationField = [[NSTextField alloc] initWithFrame:NSMakeRect(20, 185, 370, 25)];
    [_locationField setStringValue:@"/Users/sorbet/Documents"];
    [_locationField setFont:[NSFont systemFontOfSize:13]];
    [contentView addSubview:_locationField];
    
    _browseButton = [[NSButton alloc] initWithFrame:NSMakeRect(400, 185, 80, 25)];
    [_browseButton setTitle:@"Parcourir"];
    [_browseButton setTarget:self];
    [_browseButton setAction:@selector(browseForLocation:)];
    [contentView addSubview:_browseButton];
    
    // Description
    NSTextField* descFieldLabel = [[NSTextField alloc] initWithFrame:NSMakeRect(20, 150, 150, 20)];
    [descFieldLabel setStringValue:@"Description (optionnel) :"];
    [descFieldLabel setFont:[NSFont systemFontOfSize:13 weight:NSFontWeightMedium]];
    [descFieldLabel setBackgroundColor:[NSColor clearColor]];
    [descFieldLabel setBordered:NO];
    [descFieldLabel setEditable:NO];
    [contentView addSubview:descFieldLabel];
    
    NSScrollView* descScrollView = [[NSScrollView alloc] initWithFrame:NSMakeRect(20, 100, 460, 50)];
    [descScrollView setHasVerticalScroller:YES];
    [descScrollView setAutohidesScrollers:YES];
    _descriptionTextView = [[NSTextView alloc] init];
    [_descriptionTextView setString:@"Mon espace de notes personnel avec ElephantNotes V3"];
    [_descriptionTextView setFont:[NSFont systemFontOfSize:13]];
    [descScrollView setDocumentView:_descriptionTextView];
    [contentView addSubview:descScrollView];
    
    // Créer des exemples
    _createSamplesCheckbox = [[NSButton alloc] initWithFrame:NSMakeRect(20, 70, 300, 20)];
    [_createSamplesCheckbox setButtonType:NSButtonTypeSwitch];
    [_createSamplesCheckbox setTitle:@"Créer des notes d'exemple pour démarrer"];
    [_createSamplesCheckbox setState:NSControlStateValueOn];
    [_createSamplesCheckbox setFont:[NSFont systemFontOfSize:13]];
    [contentView addSubview:_createSamplesCheckbox];
    
    // Boutons
    _cancelButton = [[NSButton alloc] initWithFrame:NSMakeRect(300, 20, 80, 32)];
    [_cancelButton setTitle:@"Annuler"];
    [_cancelButton setTarget:self];
    [_cancelButton setAction:@selector(cancelCreation:)];
    [contentView addSubview:_cancelButton];
    
    _createButton = [[NSButton alloc] initWithFrame:NSMakeRect(390, 20, 90, 32)];
    [_createButton setTitle:@"Créer"];
    [_createButton setTarget:self];
    [_createButton setAction:@selector(createVault:)];
    [_createButton setKeyEquivalent:@"\r"];
    [contentView addSubview:_createButton];
    
    // Indicateur de progression
    _progressIndicator = [[NSProgressIndicator alloc] initWithFrame:NSMakeRect(20, 25, 200, 20)];
    [_progressIndicator setStyle:NSProgressIndicatorStyleBar];
    [_progressIndicator setIndeterminate:YES];
    [_progressIndicator setHidden:YES];
    [contentView addSubview:_progressIndicator];
    
    // Label de statut
    _statusLabel = [[NSTextField alloc] initWithFrame:NSMakeRect(20, 5, 300, 20)];
    [_statusLabel setStringValue:@""];
    [_statusLabel setFont:[NSFont systemFontOfSize:12]];
    [_statusLabel setTextColor:[NSColor secondaryLabelColor]];
    [_statusLabel setBackgroundColor:[NSColor clearColor]];
    [_statusLabel setBordered:NO];
    [_statusLabel setEditable:NO];
    [_statusLabel setHidden:YES];
    [contentView addSubview:_statusLabel];
}

- (void)showPopup {
    NSLog(@"🪟 Affichage de la popup de création de vault");
    [self.window makeKeyAndOrderFront:nil];
}

- (IBAction)browseForLocation:(id)sender {
    NSOpenPanel* openPanel = [NSOpenPanel openPanel];
    [openPanel setCanChooseFiles:NO];
    [openPanel setCanChooseDirectories:YES];
    [openPanel setCanCreateDirectories:YES];
    [openPanel setAllowsMultipleSelection:NO];
    [openPanel setTitle:@"Choisir l'emplacement du vault"];
    [openPanel setPrompt:@"Sélectionner"];
    
    NSString* currentLocation = [_locationField stringValue];
    if (currentLocation && [currentLocation length] > 0) {
        [openPanel setDirectoryURL:[NSURL fileURLWithPath:currentLocation]];
    }
    
    [openPanel beginSheetModalForWindow:self.window completionHandler:^(NSInteger result) {
        if (result == NSModalResponseOK) {
            NSURL* selectedURL = [[openPanel URLs] firstObject];
            NSString* selectedPath = [selectedURL path];
            [self->_locationField setStringValue:selectedPath];
        }
    }];
}

- (BOOL)validateInput {
    NSString* name = [_nameField stringValue];
    NSString* location = [_locationField stringValue];
    
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
        
        NSString* name = [self->_nameField stringValue];
        NSString* location = [self->_locationField stringValue];
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
                
                [self showSuccess:@"Vault créé avec succès !"];
                
                // Fermer immédiatement avec succès
                [self closeWithSuccess:YES vaultPath:vaultPath];
                
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

- (IBAction)cancelCreation:(id)sender {
    [self closeWithSuccess:NO vaultPath:nil];
}

- (void)closeWithSuccess:(BOOL)success vaultPath:(NSString*)vaultPath {
    // Fermer la fenêtre
    [self.window orderOut:nil];
    
    // Notifier le délégué
    if (self.delegate && [self.delegate respondsToSelector:@selector(vaultCreationPopup:didCompleteWithSuccess:vaultPath:)]) {
        [self.delegate vaultCreationPopup:self didCompleteWithSuccess:success vaultPath:vaultPath];
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

@end