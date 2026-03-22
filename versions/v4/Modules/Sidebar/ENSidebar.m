//
//  ENSidebar.m
//  ElephantNotes V4 - Module de gestion de la barre latérale
//

#import "ENSidebar.h"

@interface ENSidebar()
@property (nonatomic, assign) UIIconType activeIcon;
@end

@implementation ENSidebar

- (instancetype)initWithUIFramework:(UIFramework*)framework {
    self = [super init];
    if (self) {
        _uiFramework = framework;
        _activeIcon = UI_ICON_HOME; // Par défaut sur Dashboard
    }
    return self;
}

- (void)setup {
    if (!_uiFramework) {
        NSLog(@"❌ [ENSidebar] UIFramework non défini");
        return;
    }
    
    // Configurer les callbacks pour les événements de la sidebar
    ui_framework_set_click_handler(_uiFramework, sidebar_icon_click_handler, (__bridge void*)self);
    ui_framework_set_hover_handler(_uiFramework, sidebar_icon_hover_handler, (__bridge void*)self);
    
    // Définir l'icône active par défaut
    [self setActiveIcon:_activeIcon];
    
    NSLog(@"✅ [ENSidebar] Sidebar configurée avec succès");
}

- (void)setActiveIcon:(UIIconType)iconType {
    if (!_uiFramework) {
        NSLog(@"⚠️ [ENSidebar] UIFramework non défini");
        return;
    }
    
    _activeIcon = iconType;
    ui_framework_set_icon_active(_uiFramework, iconType);
    
    NSString* iconName = [self nameForIconType:iconType];
    NSLog(@"🎯 [ENSidebar] Icône active : %@", iconName);
}

- (void)enableIcon:(UIIconType)iconType {
    // Fonctionnalité future pour activer/désactiver des icônes
    NSLog(@"✅ [ENSidebar] Icône activée : %@", [self nameForIconType:iconType]);
}

- (void)disableIcon:(UIIconType)iconType {
    // Fonctionnalité future pour activer/désactiver des icônes  
    NSLog(@"❌ [ENSidebar] Icône désactivée : %@", [self nameForIconType:iconType]);
}

- (NSString*)nameForIconType:(UIIconType)iconType {
    switch (iconType) {
        case UI_ICON_HOME: return @"Dashboard";
        case UI_ICON_FILES: return @"Files";
        case UI_ICON_EDITOR: return @"Editor";
        case UI_ICON_SEARCH: return @"Search";
        case UI_ICON_TOOLS: return @"Tools";
        case UI_ICON_SETTINGS: return @"Settings";
        case UI_ICON_BACK: return @"Back";
        default: return @"Unknown";
    }
}

- (NSString*)emojiForIconType:(UIIconType)iconType {
    switch (iconType) {
        case UI_ICON_HOME: return @"🏠";
        case UI_ICON_FILES: return @"📁";
        case UI_ICON_EDITOR: return @"📝";
        case UI_ICON_SEARCH: return @"🔍";
        case UI_ICON_TOOLS: return @"🔧";
        case UI_ICON_SETTINGS: return @"⚙️";
        case UI_ICON_BACK: return @"⬅️";
        default: return @"❓";
    }
}

@end

// Callbacks C pour l'UI Framework
void sidebar_icon_click_handler(UIIconType iconType, void* userData) {
    if (!userData) {
        NSLog(@"❌ [ENSidebar] userData NULL dans sidebar_icon_click_handler");
        return;
    }
    
    ENSidebar* sidebar = (__bridge ENSidebar*)userData;
    NSLog(@"🔥 [ENSidebar] Clic sur icône : %@", [sidebar nameForIconType:iconType]);
    
    if (sidebar.delegate && [sidebar.delegate respondsToSelector:@selector(sidebar:didClickIcon:)]) {
        [sidebar.delegate sidebar:sidebar didClickIcon:iconType];
    }
}

void sidebar_icon_hover_handler(UIIconType iconType, bool isHovering, void* userData) {
    if (!userData) {
        return;
    }
    
    ENSidebar* sidebar = (__bridge ENSidebar*)userData;
    
    if (sidebar.delegate && [sidebar.delegate respondsToSelector:@selector(sidebar:didHoverIcon:isHovering:)]) {
        [sidebar.delegate sidebar:sidebar didHoverIcon:iconType isHovering:isHovering];
    }
    
    if (isHovering) {
        NSLog(@"🖱️ [ENSidebar] Survol : %@", [sidebar nameForIconType:iconType]);
    }
}