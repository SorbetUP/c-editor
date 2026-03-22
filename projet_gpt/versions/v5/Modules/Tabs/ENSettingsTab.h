//
//  ENSettingsTab.h
//  ElephantNotes V4 - Module Settings
//

#import "ENTabBase.h"

@interface ENSettingsTab : ENTabBase

// Méthodes spécifiques aux paramètres
- (NSArray*)getAvailableVaults;
- (NSString*)generateSettingsContent;

@end