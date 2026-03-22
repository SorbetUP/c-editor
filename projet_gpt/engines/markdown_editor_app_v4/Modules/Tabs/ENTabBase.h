//
//  ENTabBase.h
//  ElephantNotes V4 - Classe de base pour tous les onglets
//

#import <Cocoa/Cocoa.h>
#include "../ui_framework/ui_framework.h"

@protocol ENTabDelegate <NSObject>
- (void)tabDidChange:(id)sender;
- (void)tabNeedsRefresh:(id)sender;
- (void)tabContentDidSave:(id)sender withContent:(NSString*)content;
@end

@interface ENTabBase : NSObject

@property (nonatomic, assign) id<ENTabDelegate> delegate;
@property (nonatomic, readonly) NSString* tabName;
@property (nonatomic, readonly) NSString* tabIcon;
@property (nonatomic, assign) UIFramework* uiFramework;
@property (nonatomic, copy) NSString* currentVaultPath;
@property (nonatomic, copy) NSString* currentVaultName;

// Méthodes à implémenter par les sous-classes
- (instancetype)initWithName:(NSString*)name icon:(NSString*)icon;
- (NSString*)generateContent;
- (void)didBecomeActive;
- (void)didBecomeInactive;
- (void)refreshContent;

// Méthodes communes
- (void)displayContent;
- (void)showInEditor:(NSString*)content;

// Méthodes de gestion du contenu (optionnelles)
- (void)handleContentSave:(NSString*)content;

@end