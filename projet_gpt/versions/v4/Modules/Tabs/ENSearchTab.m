//
//  ENSearchTab.m
//  ElephantNotes V4 - Recherche avec interface dédiée
//

#import "ENSearchTab.h"

@interface ENFileNode : NSObject
@property (nonatomic, copy) NSString* name;
@property (nonatomic, copy) NSString* relativePath;
@property (nonatomic, assign, getter=isDirectory) BOOL directory;
@property (nonatomic, retain) NSArray* children;
- (instancetype)initWithName:(NSString*)name
                relativePath:(NSString*)relativePath
                   directory:(BOOL)directory
                    children:(NSArray*)children;
@end

@implementation ENFileNode

- (instancetype)initWithName:(NSString*)name
                relativePath:(NSString*)relativePath
                   directory:(BOOL)directory
                    children:(NSArray*)children {
    self = [super init];
    if (self) {
        _name = [name copy];
        _relativePath = [relativePath copy];
        _directory = directory;
        _children = [children retain];
    }
    return self;
}

- (void)dealloc {
    [_name release];
    [_relativePath release];
    [_children release];
    [super dealloc];
}

@end

@interface ENSearchTab () <NSOutlineViewDataSource, NSOutlineViewDelegate, NSSearchFieldDelegate> {
    NSView* _searchContainer;
    NSSearchField* _searchField;
    NSScrollView* _treeScrollView;
    NSOutlineView* _outlineView;
    NSArray* _allNodes;
    NSArray* _displayNodes;
    NSMutableSet* _expandedPaths;
    NSString* _activeQuery;
    BOOL _isActive;
    BOOL _suppressExpansionTracking;
    BOOL _suppressQuerySync;
    BOOL _observersBound;
    NSLayoutConstraint* _searchContainerLeadingConstraint;
}
@end

@implementation ENSearchTab

- (instancetype)init {
    self = [super initWithName:@"Search" icon:@"🔍"];
    if (self) {
        _expandedPaths = [[NSMutableSet alloc] init];
        _activeQuery = [@"" copy];
    }
    return self;
}

- (void)dealloc {
    NSNotificationCenter* center = [NSNotificationCenter defaultCenter];
    if (_observersBound) {
        [center removeObserver:self name:NSOutlineViewItemDidExpandNotification object:_outlineView];
        [center removeObserver:self name:NSOutlineViewItemDidCollapseNotification object:_outlineView];
    }
    [_searchContainer release];
    [_searchField release];
    [_treeScrollView release];
    [_outlineView release];
    [_allNodes release];
    [_displayNodes release];
    [_expandedPaths release];
    [_activeQuery release];
    [_searchContainerLeadingConstraint release];
    [super dealloc];
}

#pragma mark - Lifecycle

- (void)didBecomeActive {
    _isActive = YES;
    [self ensureSearchInterface];
    [self rebuildFileTree];
    [self syncSearchField];
    [self presentSearchInterface];
}

- (void)didBecomeInactive {
    _isActive = NO;
    if (_searchContainer) {
        [_searchContainer setHidden:YES];
    }
    if (self.uiFramework && self.uiFramework->editorScrollView) {
        [self.uiFramework->editorScrollView setHidden:NO];
    }
    [super didBecomeInactive];
}

- (void)refreshContent {
    [self rebuildFileTree];
    if (_isActive) {
        [self reloadOutlinePreservingState];
    }
}

- (NSString*)generateContent {
    return @"# Recherche activée\n\nLa recherche dispose maintenant d'une interface dédiée.";
}

#pragma mark - Interface Setup

- (void)ensureSearchInterface {
    if (!self.uiFramework || !self.uiFramework->containerView) {
        return;
    }
    if (!_searchContainer) {
        NSView* container = self.uiFramework->containerView;
        _searchContainer = [[NSView alloc] initWithFrame:[container bounds]];
        [_searchContainer setTranslatesAutoresizingMaskIntoConstraints:NO];
        [_searchContainer setHidden:YES];
        [_searchContainer setWantsLayer:YES];
        [_searchContainer.layer setBackgroundColor:[[NSColor windowBackgroundColor] CGColor]];
        [container addSubview:_searchContainer positioned:NSWindowBelow relativeTo:self.uiFramework->sidebarView];
        
        CGFloat sidebarWidth = self.uiFramework ? self.uiFramework->sidebarConfig.width : 60.0;
        [_searchContainerLeadingConstraint release];
        _searchContainerLeadingConstraint = [[NSLayoutConstraint constraintWithItem:_searchContainer attribute:NSLayoutAttributeLeading relatedBy:NSLayoutRelationEqual toItem:container attribute:NSLayoutAttributeLeading multiplier:1.0 constant:sidebarWidth] retain];
        NSArray* containerConstraints = [NSArray arrayWithObjects:
            _searchContainerLeadingConstraint,
            [NSLayoutConstraint constraintWithItem:_searchContainer attribute:NSLayoutAttributeTrailing relatedBy:NSLayoutRelationEqual toItem:container attribute:NSLayoutAttributeTrailing multiplier:1.0 constant:0.0],
            [NSLayoutConstraint constraintWithItem:_searchContainer attribute:NSLayoutAttributeTop relatedBy:NSLayoutRelationEqual toItem:container attribute:NSLayoutAttributeTop multiplier:1.0 constant:0.0],
            [NSLayoutConstraint constraintWithItem:_searchContainer attribute:NSLayoutAttributeBottom relatedBy:NSLayoutRelationEqual toItem:container attribute:NSLayoutAttributeBottom multiplier:1.0 constant:0.0],
            nil];
        [container addConstraints:containerConstraints];
        
        [self buildSearchControls];
    }
}

- (void)buildSearchControls {
    _searchField = [[NSSearchField alloc] initWithFrame:NSZeroRect];
    [_searchField setPlaceholderString:@"Rechercher dans vos notes et dossiers..."];
    [_searchField setSendsSearchStringImmediately:YES];
    [_searchField setSendsWholeSearchString:YES];
    [_searchField setDelegate:self];
    [_searchField setTranslatesAutoresizingMaskIntoConstraints:NO];
    [_searchContainer addSubview:_searchField];
    
    _outlineView = [[NSOutlineView alloc] initWithFrame:NSZeroRect];
    NSTableColumn* column = [[[NSTableColumn alloc] initWithIdentifier:@"FileColumn"] autorelease];
    [column setEditable:NO];
    [column setResizingMask:NSTableColumnAutoresizingMask];
    [column setTitle:@"Notes"];
    [_outlineView addTableColumn:column];
    [_outlineView setOutlineTableColumn:column];
    [_outlineView setHeaderView:nil];
    [_outlineView setDelegate:self];
    [_outlineView setDataSource:self];
    [_outlineView setRowHeight:24.0];
    if ([_outlineView respondsToSelector:@selector(setStyle:)]) {
        [_outlineView setStyle:NSTableViewStyleSourceList];
    }
    [_outlineView setTranslatesAutoresizingMaskIntoConstraints:NO];
    [_outlineView setTarget:self];
    [_outlineView setDoubleAction:@selector(handleItemDoubleClick:)];
    
    _treeScrollView = [[NSScrollView alloc] initWithFrame:NSZeroRect];
    [_treeScrollView setTranslatesAutoresizingMaskIntoConstraints:NO];
    [_treeScrollView setBorderType:NSNoBorder];
    [_treeScrollView setHasVerticalScroller:YES];
    [_treeScrollView setHasHorizontalScroller:NO];
    [_treeScrollView setDocumentView:_outlineView];
    [_searchContainer addSubview:_treeScrollView];
    
    NSArray* constraints = [NSArray arrayWithObjects:
        [NSLayoutConstraint constraintWithItem:_searchField attribute:NSLayoutAttributeLeading relatedBy:NSLayoutRelationEqual toItem:_searchContainer attribute:NSLayoutAttributeLeading multiplier:1.0 constant:20.0],
        [NSLayoutConstraint constraintWithItem:_searchField attribute:NSLayoutAttributeTrailing relatedBy:NSLayoutRelationEqual toItem:_searchContainer attribute:NSLayoutAttributeTrailing multiplier:1.0 constant:-20.0],
        [NSLayoutConstraint constraintWithItem:_searchField attribute:NSLayoutAttributeTop relatedBy:NSLayoutRelationEqual toItem:_searchContainer attribute:NSLayoutAttributeTop multiplier:1.0 constant:20.0],
        [NSLayoutConstraint constraintWithItem:_searchField attribute:NSLayoutAttributeHeight relatedBy:NSLayoutRelationEqual toItem:nil attribute:NSLayoutAttributeNotAnAttribute multiplier:1.0 constant:32.0],
        [NSLayoutConstraint constraintWithItem:_treeScrollView attribute:NSLayoutAttributeTop relatedBy:NSLayoutRelationEqual toItem:_searchField attribute:NSLayoutAttributeBottom multiplier:1.0 constant:16.0],
        [NSLayoutConstraint constraintWithItem:_treeScrollView attribute:NSLayoutAttributeLeading relatedBy:NSLayoutRelationEqual toItem:_searchContainer attribute:NSLayoutAttributeLeading multiplier:1.0 constant:12.0],
        [NSLayoutConstraint constraintWithItem:_treeScrollView attribute:NSLayoutAttributeTrailing relatedBy:NSLayoutRelationEqual toItem:_searchContainer attribute:NSLayoutAttributeTrailing multiplier:1.0 constant:-12.0],
        [NSLayoutConstraint constraintWithItem:_treeScrollView attribute:NSLayoutAttributeBottom relatedBy:NSLayoutRelationEqual toItem:_searchContainer attribute:NSLayoutAttributeBottom multiplier:1.0 constant:-12.0],
        nil];
    [_searchContainer addConstraints:constraints];
    
    if (!_observersBound) {
        NSNotificationCenter* center = [NSNotificationCenter defaultCenter];
        [center addObserver:self selector:@selector(outlineViewItemDidExpand:) name:NSOutlineViewItemDidExpandNotification object:_outlineView];
        [center addObserver:self selector:@selector(outlineViewItemDidCollapse:) name:NSOutlineViewItemDidCollapseNotification object:_outlineView];
        _observersBound = YES;
    }
}

#pragma mark - Data Handling

- (void)rebuildFileTree {
    [_allNodes release];
    _allNodes = [[self buildNodesForCurrentVault] retain];
    [self updateDisplayNodes];
    [self reloadOutlinePreservingState];
}

- (NSArray*)buildNodesForCurrentVault {
    if (!self.currentVaultPath || [self.currentVaultPath length] == 0) {
        return [NSArray array];
    }
    NSString* notesPath = [self.currentVaultPath stringByAppendingPathComponent:@"Notes"];
    BOOL isDirectory = NO;
    if (![[NSFileManager defaultManager] fileExistsAtPath:notesPath isDirectory:&isDirectory] || !isDirectory) {
        return [NSArray array];
    }
    NSFileManager* fileManager = [NSFileManager defaultManager];
    return [self nodesAtPath:notesPath relativePath:@"" fileManager:fileManager];
}

- (NSArray*)nodesAtPath:(NSString*)path relativePath:(NSString*)relativePath fileManager:(NSFileManager*)fileManager {
    NSError* error = nil;
    NSArray* contents = [fileManager contentsOfDirectoryAtPath:path error:&error];
    if (!contents) {
        return [NSArray array];
    }
    NSMutableArray* directories = [NSMutableArray array];
    NSMutableArray* files = [NSMutableArray array];
    for (NSString* itemName in contents) {
        if ([itemName hasPrefix:@"."]) {
            continue; // Ignorer les fichiers cachés
        }
        NSString* fullPath = [path stringByAppendingPathComponent:itemName];
        BOOL isDir = NO;
        if (![fileManager fileExistsAtPath:fullPath isDirectory:&isDir]) {
            continue;
        }
        NSString* newRelative = [relativePath length] > 0 ? [relativePath stringByAppendingPathComponent:itemName] : itemName;
        if (isDir) {
            NSArray* childNodes = [self nodesAtPath:fullPath relativePath:newRelative fileManager:fileManager];
            ENFileNode* node = [[[ENFileNode alloc] initWithName:itemName
                                                   relativePath:newRelative
                                                      directory:YES
                                                       children:childNodes] autorelease];
            [directories addObject:node];
        } else if ([[[itemName pathExtension] lowercaseString] isEqualToString:@"md"]) {
            ENFileNode* node = [[[ENFileNode alloc] initWithName:itemName
                                                   relativePath:newRelative
                                                      directory:NO
                                                       children:[NSArray array]] autorelease];
            [files addObject:node];
        }
    }
    NSSortDescriptor* sort = [[[NSSortDescriptor alloc] initWithKey:@"name" ascending:YES selector:@selector(caseInsensitiveCompare:)] autorelease];
    [directories sortUsingDescriptors:[NSArray arrayWithObject:sort]];
    [files sortUsingDescriptors:[NSArray arrayWithObject:sort]];
    NSMutableArray* ordered = [NSMutableArray arrayWithCapacity:[directories count] + [files count]];
    [ordered addObjectsFromArray:directories];
    [ordered addObjectsFromArray:files];
    return ordered;
}

- (void)updateDisplayNodes {
    NSArray* nodes = nil;
    if (_activeQuery && [_activeQuery length] > 0) {
        nodes = [self filteredNodes:_allNodes query:_activeQuery];
    } else {
        nodes = _allNodes ?: [NSArray array];
    }
    [_displayNodes release];
    _displayNodes = [nodes retain];
}

- (NSArray*)filteredNodes:(NSArray*)source query:(NSString*)query {
    if (!source || [source count] == 0) {
        return [NSArray array];
    }
    NSMutableArray* result = [NSMutableArray array];
    for (ENFileNode* node in source) {
        BOOL nameMatches = (query && [node.name rangeOfString:query options:NSCaseInsensitiveSearch].location != NSNotFound);
        NSArray* filteredChildren = nil;
        if (node.isDirectory) {
            filteredChildren = [self filteredNodes:node.children query:query];
        }
        BOOL includeNode = nameMatches || (filteredChildren && [filteredChildren count] > 0);
        if (includeNode) {
            NSArray* childrenToUse = nil;
            if (node.isDirectory) {
                if (nameMatches && (!filteredChildren || [filteredChildren count] == 0)) {
                    childrenToUse = node.children;
                } else {
                    childrenToUse = filteredChildren;
                }
            } else {
                childrenToUse = [NSArray array];
            }
            ENFileNode* copyNode = [[[ENFileNode alloc] initWithName:node.name
                                                        relativePath:node.relativePath
                                                           directory:node.isDirectory
                                                            children:childrenToUse] autorelease];
            [result addObject:copyNode];
        }
    }
    return [NSArray arrayWithArray:result];
}

#pragma mark - Presentation Helpers

- (void)presentSearchInterface {
    if (!self.uiFramework) {
        return;
    }
    if (_searchContainerLeadingConstraint) {
        CGFloat sidebarWidth = self.uiFramework->sidebarConfig.width;
        [_searchContainerLeadingConstraint setConstant:sidebarWidth];
    }
    if (self.uiFramework->editorScrollView) {
        [self.uiFramework->editorScrollView setHidden:YES];
    }
    if (_searchContainer) {
        [_searchContainer setHidden:NO];
    }
    [self reloadOutlinePreservingState];
    [self focusSearchField];
}

- (void)reloadOutlinePreservingState {
    if (!_outlineView) {
        return;
    }
    [_outlineView reloadData];
    if (_activeQuery && [_activeQuery length] > 0) {
        [self autoExpandAll];
    } else {
        [self applyExpansionState];
    }
}

- (void)applyExpansionState {
    if (!_outlineView) {
        return;
    }
    _suppressExpansionTracking = YES;
    for (NSString* path in _expandedPaths) {
        ENFileNode* node = [self nodeForPath:path inNodes:_displayNodes];
        if (node) {
            [_outlineView expandItem:node expandChildren:NO];
        }
    }
    _suppressExpansionTracking = NO;
}

- (void)autoExpandAll {
    if (!_outlineView) {
        return;
    }
    _suppressExpansionTracking = YES;
    for (ENFileNode* node in _displayNodes) {
        if (node.isDirectory) {
            [_outlineView expandItem:node expandChildren:YES];
        }
    }
    _suppressExpansionTracking = NO;
}

- (void)focusSearchField {
    if (!self.uiFramework || !self.uiFramework->window || !_searchField) {
        return;
    }
    _suppressQuerySync = YES;
    [_searchField setStringValue:_activeQuery ?: @""];
    _suppressQuerySync = NO;
    [self.uiFramework->window makeFirstResponder:_searchField];
    [_searchField selectText:nil];
}

- (void)syncSearchField {
    if (!_searchField) {
        return;
    }
    _suppressQuerySync = YES;
    [_searchField setStringValue:_activeQuery ?: @""];
    _suppressQuerySync = NO;
}

#pragma mark - Outline Helpers

- (ENFileNode*)nodeForPath:(NSString*)path inNodes:(NSArray*)nodes {
    for (ENFileNode* node in nodes) {
        if ([node.relativePath isEqualToString:path]) {
            return node;
        }
        if (node.isDirectory) {
            ENFileNode* found = [self nodeForPath:path inNodes:node.children];
            if (found) {
                return found;
            }
        }
    }
    return nil;
}

- (NSString*)fullPathForNode:(ENFileNode*)node {
    if (!self.currentVaultPath || !node) {
        return nil;
    }
    NSString* notesRoot = [self.currentVaultPath stringByAppendingPathComponent:@"Notes"];
    return [notesRoot stringByAppendingPathComponent:node.relativePath];
}

#pragma mark - Actions

- (void)handleItemDoubleClick:(id)sender {
    NSInteger row = [_outlineView clickedRow];
    if (row < 0) {
        return;
    }
    ENFileNode* node = [_outlineView itemAtRow:row];
    if (!node || node.isDirectory) {
        return;
    }
    NSString* path = [self fullPathForNode:node];
    if (!path) {
        return;
    }
    NSError* error = nil;
    NSString* content = [NSString stringWithContentsOfFile:path encoding:NSUTF8StringEncoding error:&error];
    if (!error && content && self.uiFramework) {
        ui_framework_set_editor_content(self.uiFramework, content);
        NSLog(@"📄 [ENSearchTab] Ouverture de %@", node.relativePath);
    } else if (error) {
        NSLog(@"⚠️ [ENSearchTab] Impossible de charger %@ : %@", node.relativePath, error);
    }
}

#pragma mark - NSSearchFieldDelegate

- (void)controlTextDidChange:(NSNotification *)notification {
    if ([notification object] != _searchField || _suppressQuerySync) {
        return;
    }
    NSString* query = [[_searchField stringValue] copy];
    [self updateQuery:query];
    [query release];
}

- (void)updateQuery:(NSString*)query {
    NSString* sanitized = query ?: @"";
    if ([_activeQuery isEqualToString:sanitized]) {
        return;
    }
    [_activeQuery release];
    _activeQuery = [sanitized copy];
    [self updateDisplayNodes];
    [self reloadOutlinePreservingState];
}

#pragma mark - NSOutlineView Data Source

- (NSInteger)outlineView:(NSOutlineView *)outlineView numberOfChildrenOfItem:(id)item {
    ENFileNode* node = (ENFileNode*)item;
    if (!node) {
        return [_displayNodes count];
    }
    return [node.children count];
}

- (id)outlineView:(NSOutlineView *)outlineView child:(NSInteger)index ofItem:(id)item {
    ENFileNode* node = (ENFileNode*)item;
    if (!node) {
        return [_displayNodes objectAtIndex:index];
    }
    return [node.children objectAtIndex:index];
}

- (BOOL)outlineView:(NSOutlineView *)outlineView isItemExpandable:(id)item {
    ENFileNode* node = (ENFileNode*)item;
    return node.isDirectory && [node.children count] > 0;
}

#pragma mark - NSOutlineView Delegate

- (NSView*)outlineView:(NSOutlineView *)outlineView viewForTableColumn:(NSTableColumn *)tableColumn item:(id)item {
    ENFileNode* node = (ENFileNode*)item;
    NSTableCellView* cell = [outlineView makeViewWithIdentifier:@"FileCell" owner:self];
    if (!cell) {
        cell = [[[NSTableCellView alloc] initWithFrame:NSMakeRect(0, 0, 100, 24)] autorelease];
        [cell setIdentifier:@"FileCell"];
        NSTextField* textField = [[[NSTextField alloc] initWithFrame:NSZeroRect] autorelease];
        [textField setBordered:NO];
        [textField setEditable:NO];
        [textField setBackgroundColor:[NSColor clearColor]];
        [textField setTranslatesAutoresizingMaskIntoConstraints:NO];
        [cell addSubview:textField];
        [cell setTextField:textField];
        
        NSImageView* imageView = [[[NSImageView alloc] initWithFrame:NSMakeRect(0, 0, 16, 16)] autorelease];
        [imageView setTranslatesAutoresizingMaskIntoConstraints:NO];
        [cell addSubview:imageView];
        [cell setImageView:imageView];
        
        NSArray* constraints = [NSArray arrayWithObjects:
            [NSLayoutConstraint constraintWithItem:imageView attribute:NSLayoutAttributeLeading relatedBy:NSLayoutRelationEqual toItem:cell attribute:NSLayoutAttributeLeading multiplier:1.0 constant:4.0],
            [NSLayoutConstraint constraintWithItem:imageView attribute:NSLayoutAttributeCenterY relatedBy:NSLayoutRelationEqual toItem:cell attribute:NSLayoutAttributeCenterY multiplier:1.0 constant:0.0],
            [NSLayoutConstraint constraintWithItem:imageView attribute:NSLayoutAttributeWidth relatedBy:NSLayoutRelationEqual toItem:nil attribute:NSLayoutAttributeNotAnAttribute multiplier:1.0 constant:16.0],
            [NSLayoutConstraint constraintWithItem:imageView attribute:NSLayoutAttributeHeight relatedBy:NSLayoutRelationEqual toItem:nil attribute:NSLayoutAttributeNotAnAttribute multiplier:1.0 constant:16.0],
            [NSLayoutConstraint constraintWithItem:textField attribute:NSLayoutAttributeLeading relatedBy:NSLayoutRelationEqual toItem:imageView attribute:NSLayoutAttributeTrailing multiplier:1.0 constant:6.0],
            [NSLayoutConstraint constraintWithItem:textField attribute:NSLayoutAttributeTrailing relatedBy:NSLayoutRelationEqual toItem:cell attribute:NSLayoutAttributeTrailing multiplier:1.0 constant:-4.0],
            [NSLayoutConstraint constraintWithItem:textField attribute:NSLayoutAttributeCenterY relatedBy:NSLayoutRelationEqual toItem:cell attribute:NSLayoutAttributeCenterY multiplier:1.0 constant:0.0],
            nil];
        [cell addConstraints:constraints];
    }
    if (node.isDirectory) {
        cell.imageView.image = [NSImage imageNamed:NSImageNameFolder];
    } else {
        cell.imageView.image = [NSImage imageNamed:NSImageNameMultipleDocuments];
    }
    if (_activeQuery && [_activeQuery length] > 0) {
        NSMutableAttributedString* attributed = [[NSMutableAttributedString alloc] initWithString:node.name?:@""];
        NSRange matchRange = [node.name rangeOfString:_activeQuery options:NSCaseInsensitiveSearch];
        if (matchRange.location != NSNotFound) {
            [attributed addAttribute:NSForegroundColorAttributeName value:[NSColor systemBlueColor] range:matchRange];
            NSFont* boldFont = [NSFont boldSystemFontOfSize:[[cell.textField font] pointSize]];
            if (boldFont) {
                [attributed addAttribute:NSFontAttributeName value:boldFont range:matchRange];
            }
        }
        [cell.textField setAttributedStringValue:attributed];
        [attributed release];
    } else {
        [cell.textField setStringValue:node.name ?: @""];
    }
    return cell;
}

- (void)outlineViewItemDidExpand:(NSNotification *)notification {
    if (_suppressExpansionTracking) {
        return;
    }
    ENFileNode* node = [[notification userInfo] objectForKey:@"NSObject"];
    if ([node isKindOfClass:[ENFileNode class]] && node.relativePath) {
        [_expandedPaths addObject:node.relativePath];
    }
}

- (void)outlineViewItemDidCollapse:(NSNotification *)notification {
    if (_suppressExpansionTracking) {
        return;
    }
    ENFileNode* node = [[notification userInfo] objectForKey:@"NSObject"];
    if ([node isKindOfClass:[ENFileNode class]] && node.relativePath) {
        [_expandedPaths removeObject:node.relativePath];
    }
}

@end
