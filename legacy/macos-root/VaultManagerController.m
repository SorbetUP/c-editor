// VaultManagerController.m - Implémentation de la gestion des vaults
#import "VaultManagerController.h"
#import "VaultSetupController.h"

@implementation VaultManagerController

- (instancetype)init {
    self = [super init];
    if (self) {
        _registry = NULL;
        _vaultInfos = [[NSMutableArray alloc] init];
        _selectedVaultIndex = -1;
        
        vault_manager_init();
    }
    return self;
}

- (void)loadWindow {
    // Créer la fenêtre de gestion des vaults
    NSRect windowFrame = NSMakeRect(0, 0, 700, 500);
    NSWindow* window = [[NSWindow alloc] 
        initWithContentRect:windowFrame
        styleMask:(NSWindowStyleMaskTitled | NSWindowStyleMaskClosable | NSWindowStyleMaskResizable)
        backing:NSBackingStoreBuffered 
        defer:NO];
    
    [window setTitle:@"Gestionnaire de Vaults"];
    [window center];
    [window setLevel:NSFloatingWindowLevel];
    [window setMovable:YES];
    [window setMinSize:NSMakeSize(600, 400)];
    
    self.window = window;
    [self setupUI];
    [self refreshVaultList];
}

- (void)setupUI {
    NSView* contentView = [self.window contentView];
    
    // Titre
    NSTextField* titleLabel = [[NSTextField alloc] initWithFrame:NSMakeRect(30, 450, 640, 30)];
    [titleLabel setStringValue:@"📁 Gestionnaire de Vaults ElephantNotes"];
    [titleLabel setFont:[NSFont systemFontOfSize:18 weight:NSFontWeightBold]];
    [titleLabel setTextColor:[NSColor labelColor]];
    [titleLabel setBackgroundColor:[NSColor clearColor]];
    [titleLabel setBordered:NO];
    [titleLabel setEditable:NO];
    [titleLabel setAlignment:NSTextAlignmentCenter];
    [contentView addSubview:titleLabel];
    
    // Liste des vaults
    NSScrollView* scrollView = [[NSScrollView alloc] initWithFrame:NSMakeRect(30, 200, 450, 220)];
    [scrollView setHasVerticalScroller:YES];
    [scrollView setAutohidesScrollers:YES];
    [scrollView setBorderType:NSBezelBorder];
    [scrollView setAutoresizingMask:NSViewWidthSizable | NSViewHeightSizable];
    
    _vaultsTableView = [[NSTableView alloc] init];
    [_vaultsTableView setDataSource:self];
    [_vaultsTableView setDelegate:self];
    [_vaultsTableView setAllowsMultipleSelection:NO];
    [_vaultsTableView setRowSizeStyle:NSTableViewRowSizeStyleMedium];
    
    // Colonnes
    NSTableColumn* nameColumn = [[NSTableColumn alloc] initWithIdentifier:@"name"];
    [nameColumn setTitle:@"Nom du Vault"];
    [nameColumn setWidth:150];
    [_vaultsTableView addTableColumn:nameColumn];
    
    NSTableColumn* pathColumn = [[NSTableColumn alloc] initWithIdentifier:@"path"];
    [pathColumn setTitle:@"Emplacement"];
    [pathColumn setWidth:250];
    [_vaultsTableView addTableColumn:pathColumn];
    
    NSTableColumn* statusColumn = [[NSTableColumn alloc] initWithIdentifier:@"status"];
    [statusColumn setTitle:@"Statut"];
    [statusColumn setWidth:80];
    [_vaultsTableView addTableColumn:statusColumn];
    
    [scrollView setDocumentView:_vaultsTableView];
    [contentView addSubview:scrollView];
    
    // Boutons de gestion
    _addVaultButton = [[NSButton alloc] initWithFrame:NSMakeRect(500, 380, 150, 32)];
    [_addVaultButton setTitle:@"Ajouter Vault..."];
    [_addVaultButton setTarget:self];
    [_addVaultButton setAction:@selector(addVault:)];
    [contentView addSubview:_addVaultButton];
    
    _removeVaultButton = [[NSButton alloc] initWithFrame:NSMakeRect(500, 340, 150, 32)];
    [_removeVaultButton setTitle:@"Supprimer"];
    [_removeVaultButton setTarget:self];
    [_removeVaultButton setAction:@selector(removeVault:)];
    [_removeVaultButton setEnabled:NO];
    [contentView addSubview:_removeVaultButton];
    
    _setDefaultButton = [[NSButton alloc] initWithFrame:NSMakeRect(500, 300, 150, 32)];
    [_setDefaultButton setTitle:@"Définir par défaut"];
    [_setDefaultButton setTarget:self];
    [_setDefaultButton setAction:@selector(setAsDefault:)];
    [_setDefaultButton setEnabled:NO];
    [contentView addSubview:_setDefaultButton];
    
    _openVaultButton = [[NSButton alloc] initWithFrame:NSMakeRect(500, 260, 150, 32)];
    [_openVaultButton setTitle:@"Ouvrir dossier"];
    [_openVaultButton setTarget:self];
    [_openVaultButton setAction:@selector(openVaultLocation:)];
    [_openVaultButton setEnabled:NO];
    [contentView addSubview:_openVaultButton];
    
    // Informations du vault sélectionné
    NSTextField* infoTitleLabel = [[NSTextField alloc] initWithFrame:NSMakeRect(30, 170, 200, 20)];
    [infoTitleLabel setStringValue:@"Informations du vault :"];
    [infoTitleLabel setFont:[NSFont systemFontOfSize:14 weight:NSFontWeightMedium]];
    [infoTitleLabel setBackgroundColor:[NSColor clearColor]];
    [infoTitleLabel setBordered:NO];
    [infoTitleLabel setEditable:NO];
    [contentView addSubview:infoTitleLabel];
    
    _vaultInfoLabel = [[NSTextField alloc] initWithFrame:NSMakeRect(30, 150, 620, 20)];
    [_vaultInfoLabel setStringValue:@"Sélectionnez un vault pour voir ses informations"];
    [_vaultInfoLabel setFont:[NSFont systemFontOfSize:12]];
    [_vaultInfoLabel setTextColor:[NSColor secondaryLabelColor]];
    [_vaultInfoLabel setBackgroundColor:[NSColor clearColor]];
    [_vaultInfoLabel setBordered:NO];
    [_vaultInfoLabel setEditable:NO];
    [contentView addSubview:_vaultInfoLabel];
    
    _vaultPathLabel = [[NSTextField alloc] initWithFrame:NSMakeRect(30, 130, 620, 20)];
    [_vaultPathLabel setStringValue:@""];
    [_vaultPathLabel setFont:[NSFont systemFontOfSize:11]];
    [_vaultPathLabel setTextColor:[NSColor tertiaryLabelColor]];
    [_vaultPathLabel setBackgroundColor:[NSColor clearColor]];
    [_vaultPathLabel setBordered:NO];
    [_vaultPathLabel setEditable:NO];
    [contentView addSubview:_vaultPathLabel];
    
    _vaultStatsLabel = [[NSTextField alloc] initWithFrame:NSMakeRect(30, 110, 620, 20)];
    [_vaultStatsLabel setStringValue:@""];
    [_vaultStatsLabel setFont:[NSFont systemFontOfSize:11]];
    [_vaultStatsLabel setTextColor:[NSColor secondaryLabelColor]];
    [_vaultStatsLabel setBackgroundColor:[NSColor clearColor]];
    [_vaultStatsLabel setBordered:NO];
    [_vaultStatsLabel setEditable:NO];
    [contentView addSubview:_vaultStatsLabel];
    
    // Boutons de contrôle
    _closeButton = [[NSButton alloc] initWithFrame:NSMakeRect(580, 30, 80, 32)];
    [_closeButton setTitle:@"Fermer"];
    [_closeButton setTarget:self];
    [_closeButton setAction:@selector(closeManager:)];
    [contentView addSubview:_closeButton];
    
    NSButton* switchVaultButton = [[NSButton alloc] initWithFrame:NSMakeRect(460, 30, 110, 32)];
    [switchVaultButton setTitle:@"Changer vault"];
    [switchVaultButton setTarget:self];
    [switchVaultButton setAction:@selector(switchToSelectedVault:)];
    [switchVaultButton setKeyEquivalent:@"\r"];
    [contentView addSubview:switchVaultButton];
}

- (void)showVaultManager {
    [self loadWindow];
    [self.window makeKeyAndOrderFront:nil];
    [NSApp runModalForWindow:self.window];
}

- (void)refreshVaultList {
    [_vaultInfos removeAllObjects];
    
    if (_registry) {
        vault_registry_free(_registry);
    }
    
    VaultResult result = vault_registry_load(&_registry);
    if (result == VAULT_SUCCESS && _registry) {
        for (int i = 0; i < _registry->count; i++) {
            VaultInfo* info = &_registry->vaults[i];
            
            NSMutableDictionary* vaultDict = [[NSMutableDictionary alloc] init];
            [vaultDict setObject:[NSString stringWithUTF8String:info->config.name] forKey:@"name"];
            [vaultDict setObject:[NSString stringWithUTF8String:info->config.path] forKey:@"path"];
            
            // Vérifier si c'est le vault par défaut
            BOOL isDefault = NO;
            if (_registry->default_vault_path) {
                isDefault = (strcmp(info->config.path, _registry->default_vault_path) == 0);
            }
            [vaultDict setObject:isDefault ? @"Par défaut" : @"" forKey:@"status"];
            
            [_vaultInfos addObject:vaultDict];
        }
    }
    
    [_vaultsTableView reloadData];
    [self updateVaultInfo];
}

- (void)updateVaultInfo {
    if (_selectedVaultIndex < 0 || _selectedVaultIndex >= [_vaultInfos count]) {
        [_vaultInfoLabel setStringValue:@"Sélectionnez un vault pour voir ses informations"];
        [_vaultPathLabel setStringValue:@""];
        [_vaultStatsLabel setStringValue:@""];
        
        [_removeVaultButton setEnabled:NO];
        [_setDefaultButton setEnabled:NO];
        [_openVaultButton setEnabled:NO];
        return;
    }
    
    NSDictionary* vaultDict = [_vaultInfos objectAtIndex:_selectedVaultIndex];
    NSString* vaultPath = [vaultDict objectForKey:@"path"];
    NSString* vaultName = [vaultDict objectForKey:@"name"];
    
    // Charger les informations détaillées
    VaultInfo* info = NULL;
    VaultResult result = vault_load([vaultPath UTF8String], &info);
    
    if (result == VAULT_SUCCESS && info) {
        NSDate* createdDate = [NSDate dateWithTimeIntervalSince1970:info->config.created];
        NSDateFormatter* formatter = [[NSDateFormatter alloc] init];
        [formatter setDateStyle:NSDateFormatterMediumStyle];
        [formatter setTimeStyle:NSDateFormatterShortStyle];
        
        [_vaultInfoLabel setStringValue:[NSString stringWithFormat:@"%@ - Créé le %@", 
                                        vaultName, [formatter stringFromDate:createdDate]]];
        [_vaultPathLabel setStringValue:vaultPath];
        
        // Statistiques (simulées pour l'instant)
        NSFileManager* fileManager = [NSFileManager defaultManager];
        NSString* notesPath = [vaultPath stringByAppendingPathComponent:@"Notes"];
        NSError* error = nil;
        NSArray* contents = [fileManager contentsOfDirectoryAtPath:notesPath error:&error];
        int noteCount = 0;
        for (NSString* file in contents) {
            if ([[file pathExtension] isEqualToString:@"md"]) {
                noteCount++;
            }
        }
        
        NSString* description = info->config.description ? 
            [NSString stringWithUTF8String:info->config.description] : @"Aucune description";
        [_vaultStatsLabel setStringValue:[NSString stringWithFormat:@"%d notes • %@", 
                                         noteCount, description]];
        
        vault_free_info(info);
        free(info);
    } else {
        [_vaultInfoLabel setStringValue:[NSString stringWithFormat:@"%@ - Erreur de chargement", vaultName]];
        [_vaultPathLabel setStringValue:vaultPath];
        [_vaultStatsLabel setStringValue:@"Impossible de charger les statistiques"];
    }
    
    [_removeVaultButton setEnabled:YES];
    [_setDefaultButton setEnabled:YES];
    [_openVaultButton setEnabled:YES];
}

// Table View Data Source
- (NSInteger)numberOfRowsInTableView:(NSTableView *)tableView {
    return [_vaultInfos count];
}

- (id)tableView:(NSTableView *)tableView objectValueForTableColumn:(NSTableColumn *)tableColumn row:(NSInteger)row {
    if (row < 0 || row >= [_vaultInfos count]) return @"";
    
    NSDictionary* vaultDict = [_vaultInfos objectAtIndex:row];
    return [vaultDict objectForKey:[tableColumn identifier]];
}

// Table View Delegate
- (void)tableViewSelectionDidChange:(NSNotification *)notification {
    _selectedVaultIndex = [_vaultsTableView selectedRow];
    [self updateVaultInfo];
}

- (void)selectVaultAtIndex:(NSInteger)index {
    if (index >= 0 && index < [_vaultInfos count]) {
        [_vaultsTableView selectRowIndexes:[NSIndexSet indexSetWithIndex:index] byExtendingSelection:NO];
        _selectedVaultIndex = index;
        [self updateVaultInfo];
    }
}

- (IBAction)addVault:(id)sender {
    VaultSetupController* setupController = [[VaultSetupController alloc] init];
    
    VaultManagerController* selfRef = self;
    setupController.completionHandler = ^(BOOL success, NSString* vaultPath) {
        if (success && vaultPath) {
            // Rafraîchir la liste après ajout
            [selfRef refreshVaultList];
        }
    };
    
    [setupController showSetupWindow];
}

- (IBAction)removeVault:(id)sender {
    if (_selectedVaultIndex < 0 || _selectedVaultIndex >= [_vaultInfos count]) return;
    
    NSDictionary* vaultDict = [_vaultInfos objectAtIndex:_selectedVaultIndex];
    NSString* vaultName = [vaultDict objectForKey:@"name"];
    NSString* vaultPath = [vaultDict objectForKey:@"path"];
    
    NSAlert* alert = [[NSAlert alloc] init];
    [alert setMessageText:@"Supprimer le vault"];
    [alert setInformativeText:[NSString stringWithFormat:
        @"Êtes-vous sûr de vouloir supprimer le vault \"%@\" ?\n\n"
        @"Cette action supprimera uniquement le vault de la liste, "
        @"les fichiers sur le disque ne seront pas supprimés.", vaultName]];
    [alert addButtonWithTitle:@"Supprimer"];
    [alert addButtonWithTitle:@"Annuler"];
    [alert setAlertStyle:NSAlertStyleWarning];
    
    NSInteger response = [alert runModal];
    if (response == NSAlertFirstButtonReturn) {
        // Supprimer du registre
        VaultResult result = vault_registry_remove(_registry, [vaultPath UTF8String]);
        if (result == VAULT_SUCCESS) {
            vault_registry_save(_registry);
            [self refreshVaultList];
            
            // Si c'était le vault par défaut, définir un nouveau par défaut
            if (_registry->count > 0 && (!_registry->default_vault_path || 
                strcmp(_registry->default_vault_path, [vaultPath UTF8String]) == 0)) {
                vault_registry_set_default(_registry, _registry->vaults[0].config.path);
                vault_registry_save(_registry);
                [self refreshVaultList];
            }
        }
    }
}

- (IBAction)setAsDefault:(id)sender {
    if (_selectedVaultIndex < 0 || _selectedVaultIndex >= [_vaultInfos count]) return;
    
    NSDictionary* vaultDict = [_vaultInfos objectAtIndex:_selectedVaultIndex];
    NSString* vaultPath = [vaultDict objectForKey:@"path"];
    
    VaultResult result = vault_registry_set_default(_registry, [vaultPath UTF8String]);
    if (result == VAULT_SUCCESS) {
        vault_registry_save(_registry);
        [self refreshVaultList];
    }
}

- (IBAction)openVaultLocation:(id)sender {
    if (_selectedVaultIndex < 0 || _selectedVaultIndex >= [_vaultInfos count]) return;
    
    NSDictionary* vaultDict = [_vaultInfos objectAtIndex:_selectedVaultIndex];
    NSString* vaultPath = [vaultDict objectForKey:@"path"];
    
    [[NSWorkspace sharedWorkspace] openURL:[NSURL fileURLWithPath:vaultPath]];
}

- (IBAction)switchToSelectedVault:(id)sender {
    if (_selectedVaultIndex < 0 || _selectedVaultIndex >= [_vaultInfos count]) return;
    
    NSDictionary* vaultDict = [_vaultInfos objectAtIndex:_selectedVaultIndex];
    NSString* vaultPath = [vaultDict objectForKey:@"path"];
    
    // Définir comme vault par défaut
    vault_registry_set_default(_registry, [vaultPath UTF8String]);
    vault_registry_save(_registry);
    
    [self closeManager:sender];
    
    // Notifier le changement
    if (self.vaultChangedHandler) {
        self.vaultChangedHandler(vaultPath);
    }
}

- (IBAction)closeManager:(id)sender {
    [NSApp stopModal];
    [self.window orderOut:nil];
}

- (void)dealloc {
    if (_registry) {
        vault_registry_free(_registry);
    }
    vault_manager_cleanup();
}

@end