//
//  ENDashboardTab.h
//  ElephantNotes V4 - Module Dashboard
//

#import "ENTabBase.h"

@interface ENDashboardTab : ENTabBase

// Méthodes spécifiques au Dashboard
- (NSDictionary*)getVaultStatistics;
- (NSArray*)getRecentFiles:(int)count;
- (NSString*)generateDashboardContent;

@end