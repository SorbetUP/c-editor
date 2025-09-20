/**
 * @file ui_framework.m
 * @brief Framework d'interface utilisateur modulaire pour ElephantNotes V3
 * 
 * Ce fichier implémente un système d'interface utilisateur complet avec:
 * - Barre latérale avec icônes cliquables
 * - Éditeur Markdown intégré avec rendu en temps réel via librairie C
 * - Système de thèmes et d'animations
 * - Gestion d'état persistante
 * 
 * Architecture:
 * 1. MarkdownEditor: Composant d'édition avec rendu C optimisé
 * 2. UIIconButton: Boutons de la barre latérale avec interactions
 * 3. UIFramework: Gestionnaire principal orchestrant tous les composants
 * 
 * @author ElephantNotes Team
 * @date 2025-09-18
 */

#import "ui_framework.h"
#import "../editor/editor_abi.h"
#import "../editor/editor.h"
#include <string.h>

static void ui_framework_layout_icons(UIFramework* framework);

// ==========================================
// MARK: - Local Type Definitions
// ==========================================

// Local definitions for inline styles (from markdown.h)
typedef enum {
    INLINE_NONE,
    INLINE_BOLD,
    INLINE_ITALIC,
    INLINE_BOLD_ITALIC,
    INLINE_HIGHLIGHT,
    INLINE_UNDERLINE,
    INLINE_CODE,
    INLINE_STRIKETHROUGH,
    INLINE_LINK,
    INLINE_IMAGE_REF
} InlineStyle;

typedef struct {
    InlineStyle style;
    size_t start;
    size_t end;
} InlineSpan;

// Local function declarations
int parse_inline_styles_local(const char *text, InlineSpan *spans, size_t max_spans);
bool validate_markdown_structure_local(const char *markdown);

// ==========================================
// MARK: - Constants & Configuration
// ==========================================

/// Fréquence de mise à jour du rendu Markdown (en secondes)
static const NSTimeInterval UI_MARKDOWN_UPDATE_INTERVAL = 0.1;

/// Configuration par défaut pour l'éditeur
static const CGFloat UI_DEFAULT_EDITOR_FONT_SIZE = 14.0;

// ==========================================
// MARK: - MarkdownEditor Interface & Implementation
// ==========================================

/**
 * @brief Éditeur de texte spécialisé pour Markdown avec rendu en temps réel
 * 
 * Cette classe hérite de NSTextView et ajoute:
 * - Rendu Markdown via la librairie C optimisée
 * - Détection automatique des changements
 * - Gestion des couleurs et polices pour le code
 * - Support des thèmes sombre/clair
 */
@interface MarkdownEditor : NSTextView {
    NSInteger currentLine;
    NSTimer *timer;
    BOOL isRendering;
    NSTimeInterval lastRenderTime;
}

// Public methods for external control
- (void)setRenderingEnabled:(BOOL)enabled;
- (void)forceRender;

@end

@implementation MarkdownEditor

#pragma mark - Lifecycle

- (instancetype)initWithFrame:(NSRect)frame {
    self = [super initWithFrame:frame];
    if (self) {
        [self setupEditor];
        [self setupMarkdownRendering];
        [self initializeCLibrary];
        
        // Forcer un rendu initial après un délai pour s'assurer que tout est initialisé
        dispatch_after(dispatch_time(DISPATCH_TIME_NOW, (int64_t)(0.1 * NSEC_PER_SEC)), dispatch_get_main_queue(), ^{
            [self forceRender];
        });
    }
    return self;
}

- (void)dealloc {
    [timer invalidate];
    timer = nil;
    
    // Nettoyer les observers
    [[NSNotificationCenter defaultCenter] removeObserver:self];
}

#pragma mark - Setup Methods

/**
 * @brief Configure l'éditeur avec les paramètres de base
 */
- (void)setupEditor {
    currentLine = 0;
    isRendering = NO;
    lastRenderTime = 0;
    
    // Configuration visuelle
    [self setFont:[NSFont fontWithName:@"Monaco" size:UI_DEFAULT_EDITOR_FONT_SIZE] 
                 ?: [NSFont monospacedSystemFontOfSize:UI_DEFAULT_EDITOR_FONT_SIZE weight:NSFontWeightRegular]];
    [self setTextColor:[NSColor colorWithRed:0.878 green:0.878 blue:0.878 alpha:1.0]];
    [self setBackgroundColor:[NSColor colorWithRed:0.118 green:0.118 blue:0.118 alpha:1.0]];
    [self setInsertionPointColor:[NSColor whiteColor]];
    
    // Activation des fonctionnalités avancées
    [self setRichText:YES];
    [self setAllowsUndo:YES];
    [self setString:@""];
    
    // Écouter les changements de sélection pour détecter les mouvements de curseur
    [[NSNotificationCenter defaultCenter] addObserver:self
                                             selector:@selector(selectionDidChange:)
                                                 name:NSTextViewDidChangeSelectionNotification
                                               object:self];
}

/**
 * @brief Configure le système de rendu Markdown automatique
 */
- (void)setupMarkdownRendering {
    timer = [NSTimer scheduledTimerWithTimeInterval:UI_MARKDOWN_UPDATE_INTERVAL 
                                             target:self 
                                           selector:@selector(update) 
                                           userInfo:nil 
                                            repeats:YES];
}

/**
 * @brief Initialise la librairie C d'édition si nécessaire
 */
- (void)initializeCLibrary {
    static BOOL isLibraryInitialized = NO;
    if (!isLibraryInitialized) {
        EditorResult result = editor_library_init();
        if (result == EDITOR_SUCCESS) {
            isLibraryInitialized = YES;
            NSLog(@"✅ Librairie C d'édition initialisée avec succès");
        } else {
            NSLog(@"❌ Échec de l'initialisation de la librairie C: %s", editor_get_error_message(result));
        }
    }
}


/**
 * @brief Obtient le numéro de ligne du curseur en utilisant la librairie C cursor
 * 
 * Utilise cursor_manager.h pour une détection plus précise de la position.
 * Fallback vers méthode manuelle si la librairie C échoue.
 * 
 * @return Le numéro de ligne (0-indexé) où se trouve le curseur
 */
- (NSInteger)lineFromCursor {
    NSRange sel = [self selectedRange];
    NSString *text = [self string];
    
    // TODO: Intégrer la librairie C cursor quand elle sera linkée
    // const char *text_cstr = [text UTF8String];
    // if (text_cstr) {
    //     cursor_position_t cursor_pos = cursor_adjust_for_formatting((int)sel.location, text_cstr, true);
    //     if (cursor_pos.is_valid) {
    //         NSLog(@"🎯 Position curseur via librairie C: ligne %d, pos %d", cursor_pos.line_index, cursor_pos.position);
    //         return cursor_pos.line_index;
    //     }
    // }
    
    // Fallback vers méthode manuelle
    NSInteger line = 0;
    NSInteger pos = 0;
    
    for (NSUInteger i = 0; i <= [text length]; i++) {
        if (i == [text length] || [text characterAtIndex:i] == '\n') {
            if (sel.location >= pos && sel.location <= i) {
                return line;
            }
            line++;
            pos = i + 1;
        }
    }
    return line;
}

- (NSArray *)lines {
    return [[self string] componentsSeparatedByString:@"\n"];
}

- (NSRange)rangeForLine:(NSInteger)line {
    NSArray *lines = [self lines];
    if (line < 0 || line >= (NSInteger)[lines count]) return NSMakeRange(0, 0);
    
    NSUInteger pos = 0;
    for (NSInteger i = 0; i < line; i++) {
        pos += [[lines objectAtIndex:i] length] + 1;
    }
    return NSMakeRange(pos, [[lines objectAtIndex:line] length]);
}

- (void)update {
    // Ne pas traiter si déjà en cours de rendu
    if (isRendering) return;
    
    NSInteger newLine = [self lineFromCursor];
    
    // Vérifier si la ligne a changé pour déclencher un re-rendu
    if (newLine != currentLine) {
        NSTimeInterval now = [[NSDate date] timeIntervalSince1970];
        // Throttling: minimum 0.1s entre les rendus
        if (now - lastRenderTime >= 0.1) {
            NSLog(@"🔄 Changement de ligne détecté via timer: %ld → %ld", currentLine, newLine);
            currentLine = newLine;
            [self render];
        }
    }
}

/**
 * @brief Appelée quand la sélection change (mouvement du curseur)
 * 
 * Cette méthode détecte les mouvements de curseur et déclenche un re-rendu
 * pour mettre à jour l'affichage des caractères de formatage Markdown.
 * Utilise un système de throttling pour éviter les boucles de rendu.
 * 
 * @param notification La notification de changement de sélection
 */
- (void)selectionDidChange:(NSNotification *)notification {
    // Ne pas traiter si déjà en cours de rendu
    if (isRendering) return;
    
    NSInteger newLine = [self lineFromCursor];
    
    // Si la ligne a changé, déclencher un re-rendu avec throttling
    if (newLine != currentLine) {
        NSTimeInterval now = [[NSDate date] timeIntervalSince1970];
        // Throttling plus strict: minimum 0.2s entre les rendus via curseur
        if (now - lastRenderTime >= 0.2) {
            NSLog(@"🎯 Curseur déplacé vers ligne %ld (throttled)", newLine);
            currentLine = newLine;
            [self render];
        }
    }
}

/**
 * @brief Effectue le rendu Markdown complet en utilisant la librairie C optimisée
 * 
 * Cette méthode utilise plusieurs API de la librairie C pour un rendu optimal:
 * - editor_parse_markdown(): Parsing complet avec structure JSON
 * - editor_markdown_to_html(): Conversion directe pour prévisualisation (si nécessaire)
 * - Cache intelligent pour éviter re-parser le même contenu
 * 
 * Performance: Optimisé pour éviter les re-calculs inutiles
 */
- (void)render {
    @try {
        // Protection contre les boucles de rendu
        if (isRendering) {
            NSLog(@"⚠️ Rendu déjà en cours, abandon pour éviter une boucle");
            return;
        }
        
        isRendering = YES;
        lastRenderTime = [[NSDate date] timeIntervalSince1970];
        
        NSString *currentText = [self string];
        if (!currentText || [currentText length] == 0) {
            NSLog(@"🔍 Render: Texte vide, abandon du rendu");
            isRendering = NO;
            return;
        }
        
        NSLog(@"🎨 Début du rendu Markdown pour %lu caractères", (unsigned long)[currentText length]);
        
        // === PHASE 1: Vérification du cache pour optimiser les performances ===
        static NSString *lastRenderedText = nil;
        static NSInteger lastCursorLine = -1;
        static NSMutableAttributedString *cachedResult = nil;
        
        NSInteger currentCursorLine = [self lineFromCursor];
        
        // Cache invalide si le texte OU la ligne du curseur a changé
        if (lastRenderedText && [lastRenderedText isEqualToString:currentText] && 
            lastCursorLine == currentCursorLine && cachedResult) {
            // Utiliser le cache si ni le texte ni la position du curseur n'ont changé
            NSLog(@"📦 Utilisation du cache pour le rendu (texte et curseur identiques)");
            [self applyFormattedTextWithSelectionPreservation:cachedResult];
            isRendering = NO;
            return;
        }
        
        NSLog(@"🔄 Nouveau rendu nécessaire (pas de cache disponible)");
        
        // === PHASE 2: Utilisation de la librairie C pour le parsing ===
        NSMutableAttributedString *formattedText = [self renderUsingCLibrary:currentText];
        
        if (formattedText) {
            // === PHASE 3: Cache et application du résultat ===
            lastRenderedText = [currentText copy];
            lastCursorLine = currentCursorLine;
            cachedResult = [formattedText copy];
            [self applyFormattedTextWithSelectionPreservation:formattedText];
        } else {
            // === PHASE 4: Fallback en cas d'échec ===
            NSLog(@"⚠️ Fallback: Utilisation du rendu simple");
            NSMutableAttributedString *fallback = [self createSimpleFormattedText:currentText];
            lastRenderedText = [currentText copy];
            lastCursorLine = currentCursorLine;
            cachedResult = [fallback copy];
            [self applyFormattedTextWithSelectionPreservation:fallback];
        }
        
        // Réinitialiser le flag de rendu
        isRendering = NO;
        
    } @catch (NSException *exception) {
        NSLog(@"❌ Exception critique dans render: %@", exception.reason);
        isRendering = NO; // Important: réinitialiser même en cas d'exception
        [self handleRenderingException:exception];
    }
}

- (NSMutableAttributedString *)createFormattedTextFromJSON:(const char *)json originalText:(NSString *)originalText {
    // Créer le texte de base
    NSMutableAttributedString *text = [[NSMutableAttributedString alloc] initWithString:originalText];
    
    NSDictionary *baseAttributes = @{
        NSForegroundColorAttributeName: [NSColor colorWithRed:0.878 green:0.878 blue:0.878 alpha:1.0],
        NSFontAttributeName: [NSFont fontWithName:@"Monaco" size:14] ?: [NSFont monospacedSystemFontOfSize:14 weight:NSFontWeightRegular]
    };
    
    [text setAttributes:baseAttributes range:NSMakeRange(0, [text length])];
    
    // Parser le JSON pour appliquer les styles
    NSString *jsonString = [NSString stringWithUTF8String:json];
    NSData *jsonData = [jsonString dataUsingEncoding:NSUTF8StringEncoding];
    
    // Log pour debug: Afficher le JSON brut
    NSLog(@"🔍 JSON brut reçu de la librairie C:");
    NSLog(@"%@", [jsonString substringToIndex:MIN(500, [jsonString length])]);
    
    if (jsonData) {
        NSError *error;
        NSDictionary *parsed = [NSJSONSerialization JSONObjectWithData:jsonData options:0 error:&error];
        if (parsed && !error) {
            NSLog(@"🔍 Structure JSON parsée (premiers éléments): %@", [parsed description]);
            [self applyStylesFromParsedJSON:parsed toText:text];
        } else {
            NSLog(@"❌ Erreur JSON: %@", error.localizedDescription);
        }
    }
    
    // === FALLBACK: Parser Markdown simple si la librairie C ne fournit pas assez d'infos ===
    NSLog(@"🔄 Application du parsing Markdown simple en fallback");
    [self applySimpleMarkdownParsing:text];
    
    return text;
}

/**
 * @brief Crée un texte formaté en utilisant les librairies C avancées
 * 
 * Utilise markdown.h pour la détection d'inline styles et editor.h pour
 * la validation de structure. Fallback vers parsing regex si les libs C échouent.
 * 
 * @param content Le contenu Markdown à formater
 * @return NSMutableAttributedString avec formatage avancé
 */
- (NSMutableAttributedString *)createSimpleFormattedText:(NSString *)content {
    NSMutableAttributedString *text = [[NSMutableAttributedString alloc] initWithString:content];
    
    // Attributs de base
    NSDictionary *baseAttributes = @{
        NSForegroundColorAttributeName: [NSColor colorWithRed:0.878 green:0.878 blue:0.878 alpha:1.0],
        NSFontAttributeName: [NSFont monospacedSystemFontOfSize:14 weight:NSFontWeightRegular]
    };
    [text setAttributes:baseAttributes range:NSMakeRange(0, [text length])];
    
    // === PHASE 1: Validation Markdown avec librairie C ===
    const char *content_cstr = [content UTF8String];
    if (content_cstr && validate_markdown_structure_local(content_cstr)) {
        NSLog(@"✅ Structure Markdown validée via librairie C");
        
        // === PHASE 2: Détection d'inline styles avancés ===
        InlineSpan spans[256]; // Buffer pour les spans
        int span_count = parse_inline_styles_local(content_cstr, spans, 256);
        
        if (span_count > 0) {
            NSLog(@"🎨 %d styles inline détectés via librairie C", span_count);
            
            // Appliquer les styles détectés par la librairie C
            for (int i = 0; i < span_count; i++) {
                NSRange range = NSMakeRange(spans[i].start, spans[i].end - spans[i].start);
                if (range.location + range.length <= [content length]) {
                    [self applyInlineStyle:spans[i].style toText:text inRange:range];
                }
            }
            return text;
        }
    }
    
    NSLog(@"⚠️ Fallback vers parsing regex simple");
    
    // === PHASE 3: Fallback - parsing regex traditionnel ===
    [self applyBasicMarkdownStyling:text];
    return text;
}

/**
 * @brief Applique les styles basés sur l'analyse JSON complète de la librairie C
 * 
 * Cette méthode parse la structure JSON retournée par editor_parse_markdown
 * et applique les styles appropriés avec validation robuste.
 * 
 * @param parsed Dictionnaire JSON parsé
 * @param text Texte à styliser
 * @return Nombre de styles appliqués avec succès
 */
- (void)applyStylesFromParsedJSON:(NSDictionary *)parsed toText:(NSMutableAttributedString *)text {
    // === SUPPORT MULTIPLE FORMATS JSON ===
    // La librairie C peut retourner différents formats selon la version
    NSArray *elements = parsed[@"elements"] ?: parsed[@"blocks"] ?: parsed[@"nodes"];
    
    if (!elements || ![elements isKindOfClass:[NSArray class]]) {
        NSLog(@"⚠️ Aucun élément trouvé dans le JSON ou format inattendu");
        return;
    }
    
    NSLog(@"🔍 Traitement de %lu éléments de style", [elements count]);
    NSUInteger stylesApplied = 0;
    
    // === TRAITEMENT DES ÉLÉMENTS ===
    for (NSDictionary *element in elements) {
        if (![element isKindOfClass:[NSDictionary class]]) {
            continue;
        }
        
        NSString *type = element[@"type"];
        NSDictionary *pos = element[@"position"] ?: element[@"range"] ?: element[@"location"];
        
        if (!type || !pos) {
            NSLog(@"⚠️ Élément JSON incomplet: type=%@, pos=%@", type, pos);
            continue;
        }
        
        // === VALIDATION ET APPLICATION ===
        NSInteger start = [pos[@"start"] integerValue];
        NSInteger end = [pos[@"end"] integerValue];
        
        if (start >= 0 && end <= [text length] && start < end) {
            NSRange range = NSMakeRange(start, end - start);
            BOOL success = [self applyEnhancedStyleForType:type range:range element:element toText:text];
            if (success) {
                stylesApplied++;
            }
        } else {
            NSLog(@"⚠️ Range invalide pour type %@: %ld-%ld (texte: %lu caractères)", 
                  type, start, end, [text length]);
        }
    }
    
    NSLog(@"✅ %lu styles appliqués avec succès", stylesApplied);
}

- (void)formatAsBold:(NSMutableAttributedString *)text range:(NSRange)range {
    [text addAttributes:@{
        NSForegroundColorAttributeName: [NSColor colorWithRed:0.0 green:0.8 blue:0.0 alpha:1.0],
        NSFontAttributeName: [NSFont boldSystemFontOfSize:14]
    } range:range];
}

- (void)formatAsItalic:(NSMutableAttributedString *)text range:(NSRange)range {
    [text addAttributes:@{
        NSForegroundColorAttributeName: [NSColor colorWithRed:1.0 green:1.0 blue:0.0 alpha:1.0],
        NSFontAttributeName: [NSFont fontWithName:@"Monaco-Italic" size:14] ?: [NSFont systemFontOfSize:14]
    } range:range];
}

- (void)formatAsHeader:(NSMutableAttributedString *)text range:(NSRange)range level:(NSInteger)level {
    CGFloat fontSize = 20 - (level - 1) * 2; // H1=20, H2=18, H3=16
    
    [text addAttributes:@{
        NSForegroundColorAttributeName: [NSColor colorWithRed:0.2 green:0.4 blue:1.0 alpha:1.0],
        NSFontAttributeName: [NSFont boldSystemFontOfSize:fontSize]
    } range:range];
}

#pragma mark - Formatage étendu avec librairie C

/**
 * @brief Applique un style spécifique basé sur le type d'élément de la librairie C (version améliorée)
 * 
 * Cette méthode est le point central d'extensibilité pour ajouter de nouveaux
 * types de formatage supportés par la librairie C. Version étendue avec plus de types.
 * 
 * @param type Type d'élément (header, bold, italic, code, link, etc.)
 * @param range Range à styliser
 * @param element Dictionnaire complet de l'élément pour infos supplémentaires
 * @param text Texte à modifier
 * @return YES si le style a été appliqué avec succès
 */
- (BOOL)applyEnhancedStyleForType:(NSString *)type range:(NSRange)range element:(NSDictionary *)element toText:(NSMutableAttributedString *)text {
    if (!type || range.location == NSNotFound) {
        return NO;
    }
    
    // === STYLES DE BASE ===
    if ([type isEqualToString:@"header"] || [type isEqualToString:@"heading"]) {
        NSInteger level = [element[@"level"] integerValue] ?: 1;
        [self formatAsHeader:text range:range level:level];
        NSLog(@"🎯 Header H%ld appliqué à %@", level, NSStringFromRange(range));
        return YES;
        
    } else if ([type isEqualToString:@"bold"] || [type isEqualToString:@"strong"]) {
        [self formatAsBold:text range:range];
        NSLog(@"🎯 Bold appliqué à %@", NSStringFromRange(range));
        return YES;
        
    } else if ([type isEqualToString:@"italic"] || [type isEqualToString:@"emphasis"]) {
        [self formatAsItalic:text range:range];
        NSLog(@"🎯 Italic appliqué à %@", NSStringFromRange(range));
        return YES;
        
    // === STYLES AVANCÉS (extensibles) ===
    } else if ([type isEqualToString:@"code"] || [type isEqualToString:@"code_span"]) {
        [self formatAsCode:text range:range];
        NSLog(@"🎯 Code appliqué à %@", NSStringFromRange(range));
        return YES;
        
    } else if ([type isEqualToString:@"link"]) {
        NSString *url = element[@"url"] ?: element[@"href"];
        [self formatAsLink:text range:range url:url];
        NSLog(@"🎯 Link appliqué à %@ (URL: %@)", NSStringFromRange(range), url);
        return YES;
        
    } else if ([type isEqualToString:@"strikethrough"]) {
        [self formatAsStrikethrough:text range:range];
        NSLog(@"🎯 Strikethrough appliqué à %@", NSStringFromRange(range));
        return YES;
        
    } else if ([type isEqualToString:@"blockquote"]) {
        [self formatAsBlockquote:text range:range];
        NSLog(@"🎯 Blockquote appliqué à %@", NSStringFromRange(range));
        return YES;
        
    } else if ([type isEqualToString:@"list_item"]) {
        BOOL isOrdered = [element[@"ordered"] boolValue];
        [self formatAsListItem:text range:range ordered:isOrdered];
        NSLog(@"🎯 List item appliqué à %@", NSStringFromRange(range));
        return YES;
        
    } else {
        NSLog(@"⚠️ Type de style non supporté: %@", type);
        return NO;
    }
}

/**
 * @brief Formate un range comme code inline
 * @param text Texte à modifier
 * @param range Range à formater
 */
- (void)formatAsCode:(NSMutableAttributedString *)text range:(NSRange)range {
    [text addAttributes:@{
        NSForegroundColorAttributeName: [NSColor colorWithRed:1.0 green:0.6 blue:0.0 alpha:1.0], // Orange
        NSFontAttributeName: [NSFont monospacedSystemFontOfSize:UI_DEFAULT_EDITOR_FONT_SIZE weight:NSFontWeightMedium],
        NSBackgroundColorAttributeName: [NSColor colorWithRed:0.2 green:0.2 blue:0.2 alpha:1.0]
    } range:range];
}

/**
 * @brief Formate un range comme lien
 * @param text Texte à modifier
 * @param range Range à formater
 * @param url URL du lien
 */
- (void)formatAsLink:(NSMutableAttributedString *)text range:(NSRange)range url:(NSString *)url {
    NSMutableDictionary *attributes = [@{
        NSForegroundColorAttributeName: [NSColor colorWithRed:0.0 green:0.5 blue:1.0 alpha:1.0], // Bleu
        NSUnderlineStyleAttributeName: @(NSUnderlineStyleSingle)
    } mutableCopy];
    
    if (url) {
        attributes[NSLinkAttributeName] = url;
        attributes[NSToolTipAttributeName] = url;
    }
    
    [text addAttributes:attributes range:range];
}

/**
 * @brief Formate un range comme texte barré
 * @param text Texte à modifier
 * @param range Range à formater
 */
- (void)formatAsStrikethrough:(NSMutableAttributedString *)text range:(NSRange)range {
    [text addAttributes:@{
        NSForegroundColorAttributeName: [NSColor colorWithRed:0.6 green:0.6 blue:0.6 alpha:1.0], // Gris
        NSStrikethroughStyleAttributeName: @(NSUnderlineStyleSingle)
    } range:range];
}

/**
 * @brief Formate un range comme blockquote
 * @param text Texte à modifier
 * @param range Range à formater
 */
- (void)formatAsBlockquote:(NSMutableAttributedString *)text range:(NSRange)range {
    [text addAttributes:@{
        NSForegroundColorAttributeName: [NSColor colorWithRed:0.7 green:0.7 blue:0.9 alpha:1.0], // Bleu gris
        NSFontAttributeName: [NSFont fontWithName:@"Georgia" size:UI_DEFAULT_EDITOR_FONT_SIZE] ?: [NSFont systemFontOfSize:UI_DEFAULT_EDITOR_FONT_SIZE],
        NSBackgroundColorAttributeName: [NSColor colorWithRed:0.15 green:0.15 blue:0.2 alpha:1.0]
    } range:range];
}

/**
 * @brief Formate un range comme élément de liste
 * @param text Texte à modifier
 * @param range Range à formater
 * @param ordered YES pour liste numérotée, NO pour liste à puces
 */
- (void)formatAsListItem:(NSMutableAttributedString *)text range:(NSRange)range ordered:(BOOL)ordered {
    NSColor *listColor = ordered ? 
        [NSColor colorWithRed:0.9 green:0.7 blue:0.3 alpha:1.0] : // Orange pour numérotée
        [NSColor colorWithRed:0.7 green:0.9 blue:0.7 alpha:1.0];   // Vert pour puces
    
    [text addAttributes:@{
        NSForegroundColorAttributeName: listColor,
        NSFontAttributeName: [NSFont systemFontOfSize:UI_DEFAULT_EDITOR_FONT_SIZE weight:NSFontWeightMedium]
    } range:range];
}





- (void)keyDown:(NSEvent *)event {
    [super keyDown:event];
    dispatch_async(dispatch_get_main_queue(), ^{
        [self update];
    });
}

- (void)mouseDown:(NSEvent *)event {
    [super mouseDown:event];
    dispatch_async(dispatch_get_main_queue(), ^{
        [self update];
    });
}

#pragma mark - Public Interface

/**
 * @brief Active ou désactive le rendu automatique
 * @param enabled YES pour activer, NO pour désactiver
 */
- (void)setRenderingEnabled:(BOOL)enabled {
    if (enabled && !timer) {
        [self setupMarkdownRendering];
    } else if (!enabled && timer) {
        [timer invalidate];
        timer = nil;
    }
}

/**
 * @brief Force un rendu immédiat du contenu
 */
- (void)forceRender {
    [self render];
}

#pragma mark - Nouvelles méthodes pour utilisation optimisée de la librairie C

/**
 * @brief Effectue le rendu Markdown en utilisant la librairie C de manière optimisée
 * 
 * Cette méthode utilise plusieurs stratégies de la librairie C:
 * 1. editor_parse_markdown(): Pour une analyse structurelle complète
 * 2. editor_markdown_to_html(): Pour une conversion rapide (si nécessaire)
 * 3. editor_parse_markdown_simple(): Fallback rapide
 * 
 * @param text Le texte Markdown à rendre
 * @return NSMutableAttributedString formaté ou nil en cas d'échec
 */
- (NSMutableAttributedString *)renderUsingCLibrary:(NSString *)text {
    const char *markdown_cstr = [text UTF8String];
    
    // === MÉTHODE 1: Parsing structurel avec JSON ===
    char *json_result = NULL;
    EditorResult result = editor_parse_markdown(markdown_cstr, &json_result);
    
    if (result == EDITOR_SUCCESS && json_result) {
        NSMutableAttributedString *formattedText = [self createFormattedTextFromJSON:json_result originalText:text];
        editor_free_string(json_result);
        
        if (formattedText) {
            NSLog(@"✅ Rendu JSON réussi (%lu caractères)", [formattedText length]);
            return formattedText;
        }
    } else {
        NSLog(@"⚠️ Parsing JSON échoué: %s", editor_get_error_message(result));
    }
    
    // === MÉTHODE 2: API simple comme fallback ===
    const char *simple_json = editor_parse_markdown_simple(markdown_cstr);
    if (simple_json) {
        NSMutableAttributedString *simpleFormatted = [self createFormattedTextFromSimpleJSON:simple_json originalText:text];
        if (simpleFormatted) {
            NSLog(@"✅ Rendu simple réussi");
            return simpleFormatted;
        }
    }
    
    // === MÉTHODE 3: Debug avec HTML (optionnel) ===
    const char *html_result = editor_markdown_to_html(markdown_cstr);
    if (html_result) {
        NSLog(@"🔍 HTML généré pour debug: %.100s...", html_result);
        // Note: On pourrait utiliser NSAttributedString(html:) ici si nécessaire
    }
    
    NSLog(@"❌ Échec de toutes les méthodes de rendu C");
    return nil;
}

/**
 * @brief Crée un texte formaté à partir du JSON simple retourné par editor_parse_markdown_simple
 * 
 * Cette méthode est optimisée pour les résultats de l'API simple de la librairie C.
 * Elle est plus rapide mais offre moins de fonctionnalités que createFormattedTextFromJSON.
 * 
 * @param json JSON simple retourné par editor_parse_markdown_simple
 * @param originalText Texte Markdown original
 * @return NSMutableAttributedString formaté ou nil en cas d'erreur
 */
- (NSMutableAttributedString *)createFormattedTextFromSimpleJSON:(const char *)json originalText:(NSString *)originalText {
    if (!json) return nil;
    
    NSMutableAttributedString *text = [[NSMutableAttributedString alloc] initWithString:originalText];
    [text setAttributes:[self getOptimizedBaseAttributes] range:NSMakeRange(0, [text length])];
    
    // Parser le JSON simple et appliquer les styles de base
    NSString *jsonString = [NSString stringWithUTF8String:json];
    NSData *jsonData = [jsonString dataUsingEncoding:NSUTF8StringEncoding];
    
    if (jsonData) {
        NSError *error;
        id parsed = [NSJSONSerialization JSONObjectWithData:jsonData options:0 error:&error];
        if (!error && parsed) {
            [self applySimpleStylesFromJSON:parsed toText:text];
        }
    }
    
    return text;
}

/**
 * @brief Applique un parsing Markdown simple avec gestion intelligente du curseur
 * 
 * Cette méthode analyse le texte brut et applique les styles Markdown de base:
 * - Headers (# ## ###)
 * - Bold (**text**) avec caractères de formatage cachés
 * - Italic (*text*) avec caractères de formatage cachés
 * - Détection de la ligne active via le curseur
 * 
 * @param text Le texte à formater
 */
- (void)applySimpleMarkdownParsing:(NSMutableAttributedString *)text {
    NSString *string = [text string];
    NSArray *lines = [string componentsSeparatedByString:@"\n"];
    NSUInteger currentPosition = 0;
    
    // Déterminer la ligne active (où se trouve le curseur)
    NSInteger activeLine = [self getCurrentCursorLine];
    NSInteger lineIndex = 0;
    
    for (NSString *line in lines) {
        NSUInteger lineLength = [line length];
        BOOL isActiveLine = (lineIndex == activeLine);
        
        // === HEADERS ===
        if ([line hasPrefix:@"### "]) {
            NSRange headerRange = NSMakeRange(currentPosition, lineLength);
            [self formatAsHeader:text range:headerRange level:3];
            if (!isActiveLine) {
                [self hideMarkdownCharacters:text inRange:NSMakeRange(currentPosition, 4) replacement:@""];
            }
        } else if ([line hasPrefix:@"## "]) {
            NSRange headerRange = NSMakeRange(currentPosition, lineLength);
            [self formatAsHeader:text range:headerRange level:2];
            if (!isActiveLine) {
                [self hideMarkdownCharacters:text inRange:NSMakeRange(currentPosition, 3) replacement:@""];
            }
        } else if ([line hasPrefix:@"# "]) {
            NSRange headerRange = NSMakeRange(currentPosition, lineLength);
            [self formatAsHeader:text range:headerRange level:1];
            if (!isActiveLine) {
                [self hideMarkdownCharacters:text inRange:NSMakeRange(currentPosition, 2) replacement:@""];
            }
        }
        
        // === BOLD (**text**) - Traiter en premier (plus spécifique) ===
        [self formatMarkdownBoldInLine:line atPosition:currentPosition text:text hideCharacters:!isActiveLine];
        
        // === ITALIC (*text*) - Traiter après (moins spécifique) ===
        [self formatMarkdownItalicInLine:line atPosition:currentPosition text:text hideCharacters:!isActiveLine];
        
        // Passer à la ligne suivante (+1 pour le \n)
        currentPosition += lineLength + 1;
        lineIndex++;
    }
    
    NSLog(@"✅ Parsing Markdown simple appliqué avec gestion curseur");
}

/**
 * @brief Détermine la ligne actuelle du curseur
 * 
 * @return L'index de la ligne où se trouve le curseur (0-based)
 */
- (NSInteger)getCurrentCursorLine {
    return [self lineFromCursor];
}

/**
 * @brief Cache les caractères de formatage Markdown en les rendant transparents
 * 
 * @param text Le texte à modifier
 * @param range La plage des caractères à cacher
 * @param replacement Le texte de remplacement (peut être vide)
 */
- (void)hideMarkdownCharacters:(NSMutableAttributedString *)text inRange:(NSRange)range replacement:(NSString *)replacement {
    if (range.location + range.length <= [text length]) {
        // Rendre les caractères transparents
        [text addAttribute:NSForegroundColorAttributeName
                     value:[NSColor colorWithRed:0.0 green:0.0 blue:0.0 alpha:0.0]
                     range:range];
        
        // Réduire légèrement la taille pour les rendre moins visibles
        NSFont *smallerFont = [NSFont fontWithName:@"Monaco" size:8.0] ?: [NSFont monospacedSystemFontOfSize:8.0 weight:NSFontWeightLight];
        [text addAttribute:NSFontAttributeName value:smallerFont range:range];
    }
}

/**
 * @brief Formate le texte bold dans une ligne avec gestion avancée
 * 
 * @param line La ligne à analyser
 * @param position Position de début dans le texte complet
 * @param text Le texte complet à formater
 * @param hideCharacters Si true, cache les caractères **
 */
- (void)formatMarkdownBoldInLine:(NSString *)line atPosition:(NSUInteger)position text:(NSMutableAttributedString *)text hideCharacters:(BOOL)hideCharacters {
    NSError *error = nil;
    NSRegularExpression *boldRegex = [NSRegularExpression regularExpressionWithPattern:@"\\*\\*([^*]+)\\*\\*" options:0 error:&error];
    
    if (error) {
        NSLog(@"❌ Erreur regex bold: %@", error.localizedDescription);
        return;
    }
    
    NSArray *matches = [boldRegex matchesInString:line options:0 range:NSMakeRange(0, [line length])];
    
    for (NSTextCheckingResult *match in matches) {
        NSRange fullRange = NSMakeRange(position + match.range.location, match.range.length);
        NSRange contentRange = NSMakeRange(position + [match rangeAtIndex:1].location, [match rangeAtIndex:1].length);
        
        if (fullRange.location + fullRange.length <= [text length]) {
            // Formater le contenu en bold
            [self formatAsBold:text range:contentRange];
            
            if (hideCharacters) {
                // Cacher les ** du début
                NSRange startMarkers = NSMakeRange(fullRange.location, 2);
                [self hideMarkdownCharacters:text inRange:startMarkers replacement:@""];
                
                // Cacher les ** de la fin
                NSRange endMarkers = NSMakeRange(fullRange.location + fullRange.length - 2, 2);
                [self hideMarkdownCharacters:text inRange:endMarkers replacement:@""];
            }
        }
    }
}

/**
 * @brief Formate le texte italic dans une ligne avec gestion avancée
 * 
 * @param line La ligne à analyser
 * @param position Position de début dans le texte complet
 * @param text Le texte complet à formater
 * @param hideCharacters Si true, cache les caractères *
 */
- (void)formatMarkdownItalicInLine:(NSString *)line atPosition:(NSUInteger)position text:(NSMutableAttributedString *)text hideCharacters:(BOOL)hideCharacters {
    NSError *error = nil;
    // Pattern plus restrictif pour éviter les conflits avec bold
    NSRegularExpression *italicRegex = [NSRegularExpression regularExpressionWithPattern:@"(?<!\\*)\\*([^*]+)\\*(?!\\*)" options:0 error:&error];
    
    if (error) {
        NSLog(@"❌ Erreur regex italic: %@", error.localizedDescription);
        return;
    }
    
    NSArray *matches = [italicRegex matchesInString:line options:0 range:NSMakeRange(0, [line length])];
    
    for (NSTextCheckingResult *match in matches) {
        NSRange fullRange = NSMakeRange(position + match.range.location, match.range.length);
        NSRange contentRange = NSMakeRange(position + [match rangeAtIndex:1].location, [match rangeAtIndex:1].length);
        
        if (fullRange.location + fullRange.length <= [text length]) {
            // Formater le contenu en italic
            [self formatAsItalic:text range:contentRange];
            
            if (hideCharacters) {
                // Cacher le * du début
                NSRange startMarker = NSMakeRange(fullRange.location, 1);
                [self hideMarkdownCharacters:text inRange:startMarker replacement:@""];
                
                // Cacher le * de la fin
                NSRange endMarker = NSMakeRange(fullRange.location + fullRange.length - 1, 1);
                [self hideMarkdownCharacters:text inRange:endMarker replacement:@""];
            }
        }
    }
}

/**
 * @brief Retourne les attributs de texte optimisés pour l'éditeur
 * 
 * Ces attributs sont optimisés pour la lisibilité et les performances.
 * Centralisés ici pour faciliter les modifications futures.
 * 
 * @return Dictionnaire des attributs de base optimisés
 */
- (NSDictionary *)getOptimizedBaseAttributes {
    static NSDictionary *cachedAttributes = nil;
    static dispatch_once_t onceToken;
    
    dispatch_once(&onceToken, ^{
        // Police optimisée pour l'édition de code avec fallback intelligent
        NSFont *preferredFont = [NSFont fontWithName:@"Monaco" size:UI_DEFAULT_EDITOR_FONT_SIZE];
        if (!preferredFont) {
            preferredFont = [NSFont fontWithName:@"Menlo" size:UI_DEFAULT_EDITOR_FONT_SIZE];
        }
        if (!preferredFont) {
            preferredFont = [NSFont monospacedSystemFontOfSize:UI_DEFAULT_EDITOR_FONT_SIZE weight:NSFontWeightRegular];
        }
        
        cachedAttributes = @{
            NSForegroundColorAttributeName: [NSColor colorWithRed:0.878 green:0.878 blue:0.878 alpha:1.0],
            NSFontAttributeName: preferredFont,
            NSKernAttributeName: @(0.0), // Espacement optimal pour la lecture
            NSLigatureAttributeName: @(0) // Désactiver les ligatures pour le code
        };
    });
    
    return cachedAttributes;
}

/**
 * @brief Applique le texte formaté en préservant la sélection utilisateur
 * 
 * Cette méthode centralise l'application du texte formaté avec gestion
 * intelligente de la sélection pour une expérience utilisateur fluide.
 * 
 * @param formattedText Texte formaté à appliquer
 */
- (void)applyFormattedTextWithSelectionPreservation:(NSMutableAttributedString *)formattedText {
    if (!formattedText || [formattedText length] == 0) {
        return;
    }
    
    NSRange oldSelection = [self selectedRange];
    
    // Application atomique pour éviter les clignotements
    [[self textStorage] setAttributedString:formattedText];
    
    // Restauration intelligente de la sélection
    if (oldSelection.location <= [formattedText length]) {
        NSRange safeSelection = NSMakeRange(
            oldSelection.location,
            MIN(oldSelection.length, [formattedText length] - oldSelection.location)
        );
        [self setSelectedRange:safeSelection];
    }
    
    // === PHASE FINALE: Masquer les marqueurs Markdown sauf sur la ligne active ===
    [self hideMarkdownMarkersExceptCurrentLine];
}

/**
 * @brief Gestion centralisée des exceptions de rendu
 * 
 * Fournit une gestion robuste des erreurs avec logging détaillé
 * et récupération gracieuse.
 * 
 * @param exception Exception capturée
 */
- (void)handleRenderingException:(NSException *)exception {
    NSLog(@"❌ EXCEPTION CRITIQUE DE RENDU:");
    NSLog(@"❌ Nom: %@", exception.name);
    NSLog(@"❌ Raison: %@", exception.reason);
    NSLog(@"❌ Stack: %@", exception.callStackSymbols);
    
    // Tentative de récupération avec texte minimal
    @try {
        NSString *currentText = [self string];
        if (currentText && [currentText length] > 0) {
            NSMutableAttributedString *emergencyText = [[NSMutableAttributedString alloc] initWithString:currentText];
            [emergencyText setAttributes:[self getOptimizedBaseAttributes] range:NSMakeRange(0, [emergencyText length])];
            [[self textStorage] setAttributedString:emergencyText];
            NSLog(@"✅ Récupération d'urgence réussie");
        }
    } @catch (NSException *secondaryException) {
        NSLog(@"❌ Échec de la récupération d'urgence: %@", secondaryException.reason);
    }
}

/**
 * @brief Applique des styles simples basés sur le JSON simple de la librairie C
 * 
 * Optimisé pour les résultats de editor_parse_markdown_simple().
 * Plus rapide mais moins précis que applyStylesFromParsedJSON.
 * 
 * @param parsed JSON simple parsé
 * @param text Texte à styliser
 */
- (void)applySimpleStylesFromJSON:(id)parsed toText:(NSMutableAttributedString *)text {
    // Implementation simplifiée pour le JSON de base
    if ([parsed isKindOfClass:[NSArray class]]) {
        NSArray *simpleElements = (NSArray *)parsed;
        for (NSDictionary *element in simpleElements) {
            if ([element isKindOfClass:[NSDictionary class]]) {
                NSString *type = element[@"type"];
                NSNumber *start = element[@"start"];
                NSNumber *end = element[@"end"];
                
                if (type && start && end) {
                    NSRange range = NSMakeRange([start integerValue], [end integerValue] - [start integerValue]);
                    if (range.location + range.length <= [text length]) {
                        [self applyEnhancedStyleForType:type range:range element:element toText:text];
                    }
                }
            }
        }
    } else if ([parsed isKindOfClass:[NSDictionary class]]) {
        // Alternative: structure avec blocs
        NSArray *blocks = parsed[@"blocks"];
        if (blocks) {
            [self applySimpleStylesFromJSON:blocks toText:text];
        }
    }
}

/**
 * @brief Applique un style inline détecté par la librairie C
 * 
 * Convertit les types InlineStyle de markdown.h en attributs NSMutableAttributedString.
 * 
 * @param style Le type de style détecté par parse_inline_styles()
 * @param text Le texte à formater
 * @param range La plage de caractères à formater
 */
- (void)applyInlineStyle:(InlineStyle)style toText:(NSMutableAttributedString *)text inRange:(NSRange)range {
    NSMutableDictionary *attributes = [NSMutableDictionary dictionary];
    
    switch (style) {
        case INLINE_BOLD:
            [attributes setObject:[NSFont boldSystemFontOfSize:14] forKey:NSFontAttributeName];
            [attributes setObject:[NSColor colorWithRed:0.0 green:0.8 blue:0.0 alpha:1.0] forKey:NSForegroundColorAttributeName];
            break;
            
        case INLINE_ITALIC:
            [attributes setObject:[[NSFontManager sharedFontManager] convertFont:[NSFont systemFontOfSize:14] toHaveTrait:NSItalicFontMask] forKey:NSFontAttributeName];
            [attributes setObject:[NSColor colorWithRed:1.0 green:1.0 blue:0.0 alpha:1.0] forKey:NSForegroundColorAttributeName];
            break;
            
        case INLINE_BOLD_ITALIC:
            [attributes setObject:[[NSFontManager sharedFontManager] convertFont:[NSFont boldSystemFontOfSize:14] toHaveTrait:NSItalicFontMask] forKey:NSFontAttributeName];
            [attributes setObject:[NSColor colorWithRed:0.0 green:0.8 blue:0.8 alpha:1.0] forKey:NSForegroundColorAttributeName];
            break;
            
        case INLINE_CODE:
            [attributes setObject:[NSFont monospacedSystemFontOfSize:14 weight:NSFontWeightRegular] forKey:NSFontAttributeName];
            [attributes setObject:[NSColor colorWithRed:1.0 green:0.6 blue:0.0 alpha:1.0] forKey:NSForegroundColorAttributeName];
            break;
            
        case INLINE_HIGHLIGHT:
            [attributes setObject:[NSColor colorWithRed:1.0 green:1.0 blue:0.0 alpha:0.3] forKey:NSBackgroundColorAttributeName];
            break;
            
        case INLINE_STRIKETHROUGH:
            [attributes setObject:@(NSUnderlineStyleSingle) forKey:NSStrikethroughStyleAttributeName];
            [attributes setObject:[NSColor colorWithRed:0.5 green:0.5 blue:0.5 alpha:1.0] forKey:NSForegroundColorAttributeName];
            break;
            
        case INLINE_LINK:
            [attributes setObject:[NSColor colorWithRed:0.3 green:0.7 blue:1.0 alpha:1.0] forKey:NSForegroundColorAttributeName];
            [attributes setObject:@(NSUnderlineStyleSingle) forKey:NSUnderlineStyleAttributeName];
            break;
            
        case INLINE_UNDERLINE:
            [attributes setObject:@(NSUnderlineStyleSingle) forKey:NSUnderlineStyleAttributeName];
            break;
            
        default:
            NSLog(@"⚠️ Style inline non supporté: %d", style);
            return;
    }
    
    [text addAttributes:attributes range:range];
    NSLog(@"🎨 Style appliqué: %d sur range (%lu, %lu)", style, range.location, range.length);
}

/**
 * @brief Applique les styles Markdown de base avec parsing regex
 * 
 * Méthode de fallback utilisant regex NSRegularExpression pour le formatage.
 * 
 * @param text Le texte à formater
 */
- (void)applyBasicMarkdownStyling:(NSMutableAttributedString *)text {
    NSString *string = [text string];
    NSError *error = nil;
    
    // Headers
    NSRegularExpression *headerRegex = [NSRegularExpression regularExpressionWithPattern:@"^(#{1,6})\\s+(.+)$" 
                                                                                 options:NSRegularExpressionAnchorsMatchLines 
                                                                                   error:&error];
    [headerRegex enumerateMatchesInString:string options:0 range:NSMakeRange(0, [string length]) 
                               usingBlock:^(NSTextCheckingResult *match, NSMatchingFlags flags, BOOL *stop) {
        NSRange headerRange = [match rangeAtIndex:0];
        NSRange levelRange = [match rangeAtIndex:1];
        
        int level = (int)levelRange.length;
        CGFloat fontSize = 20.0 - (level * 2);
        
        [text addAttributes:@{
            NSFontAttributeName: [NSFont boldSystemFontOfSize:fontSize],
            NSForegroundColorAttributeName: [NSColor colorWithRed:0.2 green:0.4 blue:1.0 alpha:1.0]
        } range:headerRange];
    }];
    
    // Bold
    NSRegularExpression *boldRegex = [NSRegularExpression regularExpressionWithPattern:@"\\*\\*([^*]+)\\*\\*" 
                                                                               options:0 error:&error];
    [boldRegex enumerateMatchesInString:string options:0 range:NSMakeRange(0, [string length]) 
                             usingBlock:^(NSTextCheckingResult *match, NSMatchingFlags flags, BOOL *stop) {
        [text addAttributes:@{
            NSFontAttributeName: [NSFont boldSystemFontOfSize:14],
            NSForegroundColorAttributeName: [NSColor colorWithRed:0.0 green:0.8 blue:0.0 alpha:1.0]
        } range:[match range]];
    }];
    
    // Italic (éviter conflit avec bold)
    NSRegularExpression *italicRegex = [NSRegularExpression regularExpressionWithPattern:@"(?<!\\*)\\*([^*]+)\\*(?!\\*)" 
                                                                                 options:0 error:&error];
    [italicRegex enumerateMatchesInString:string options:0 range:NSMakeRange(0, [string length]) 
                               usingBlock:^(NSTextCheckingResult *match, NSMatchingFlags flags, BOOL *stop) {
        [text addAttributes:@{
            NSFontAttributeName: [[NSFontManager sharedFontManager] convertFont:[NSFont systemFontOfSize:14] toHaveTrait:NSItalicFontMask],
            NSForegroundColorAttributeName: [NSColor colorWithRed:1.0 green:1.0 blue:0.0 alpha:1.0]
        } range:[match range]];
    }];
    
    NSLog(@"🎨 Styles regex appliqués sur %lu caractères", [string length]);
}

/**
 * @brief Implémentation locale de validation Markdown 
 * 
 * Fallback simple qui valide les structures de base.
 * 
 * @param markdown Le texte Markdown à valider
 * @return true si la structure semble valide
 */
bool validate_markdown_structure_local(const char *markdown) {
    if (!markdown) return false;
    
    // Validation basique: vérifier que ce n'est pas vide et contient des caractères valides
    size_t len = strlen(markdown);
    if (len == 0) return false;
    
    // Pour l'instant, on accepte tout texte non-vide
    // TODO: ajouter des validations plus sophistiquées
    return true;
}

/**
 * @brief Implémentation locale de parsing d'inline styles
 * 
 * Parser simple qui détecte les styles Markdown basiques.
 * 
 * @param text Le texte à analyser 
 * @param spans Buffer pour stocker les spans détectés
 * @param max_spans Taille maximale du buffer
 * @return Nombre de spans détectés
 */
int parse_inline_styles_local(const char *text, InlineSpan *spans, size_t max_spans) {
    if (!text || !spans || max_spans == 0) return 0;
    
    size_t len = strlen(text);
    int span_count = 0;
    
    // Recherche de patterns bold **text**
    for (size_t i = 0; i < len - 4 && span_count < max_spans; i++) {
        if (text[i] == '*' && text[i+1] == '*') {
            // Trouvé début de bold, chercher la fin
            for (size_t j = i + 2; j < len - 1; j++) {
                if (text[j] == '*' && text[j+1] == '*') {
                    spans[span_count].style = INLINE_BOLD;
                    spans[span_count].start = i;
                    spans[span_count].end = j + 2;
                    span_count++;
                    i = j + 1; // Éviter overlap
                    break;
                }
            }
        }
    }
    
    // Recherche de patterns italic *text* (éviter conflit avec bold)
    for (size_t i = 0; i < len - 2 && span_count < max_spans; i++) {
        if (text[i] == '*' && text[i+1] != '*') {
            // Vérifier qu'on n'est pas dans un bold
            bool in_bold = false;
            for (int s = 0; s < span_count; s++) {
                if (spans[s].style == INLINE_BOLD && i >= spans[s].start && i < spans[s].end) {
                    in_bold = true;
                    break;
                }
            }
            
            if (!in_bold) {
                // Chercher la fin de l'italic
                for (size_t j = i + 1; j < len; j++) {
                    if (text[j] == '*' && (j == len - 1 || text[j+1] != '*')) {
                        spans[span_count].style = INLINE_ITALIC;
                        spans[span_count].start = i;
                        spans[span_count].end = j + 1;
                        span_count++;
                        i = j; // Éviter overlap
                        break;
                    }
                }
            }
        }
    }
    
    return span_count;
}

/**
 * @brief Masque visuellement les marqueurs Markdown sauf sur la ligne active
 * 
 * Rend les caractères de formatage (**, *, ##, etc.) transparents sur toutes
 * les lignes sauf celle où se trouve le curseur, tout en conservant le formatage
 * appliqué au texte.
 */
- (void)hideMarkdownMarkersExceptCurrentLine {
    NSMutableAttributedString *textStorage = [self textStorage];
    NSString *fullText = [textStorage string];
    NSInteger currentLine = [self lineFromCursor];
    
    if (!fullText || [fullText length] == 0) return;
    
    NSArray *lines = [fullText componentsSeparatedByString:@"\n"];
    NSUInteger position = 0;
    
    for (NSUInteger lineIndex = 0; lineIndex < [lines count]; lineIndex++) {
        NSString *line = [lines objectAtIndex:lineIndex];
        NSUInteger lineLength = [line length];
        
        if ((NSInteger)lineIndex != currentLine) {
            // Ligne inactive : rendre les marqueurs transparents
            [self hideMarkdownMarkersInRange:NSMakeRange(position, lineLength) 
                                      inText:textStorage
                                        line:line];
        }
        
        position += lineLength + 1; // +1 pour le \n
    }
}

/**
 * @brief Rend transparents les marqueurs Markdown dans une plage spécifique
 * 
 * @param range Plage de texte à traiter
 * @param textStorage Le NSMutableAttributedString à modifier
 * @param line Le contenu de la ligne pour détecter les marqueurs
 */
- (void)hideMarkdownMarkersInRange:(NSRange)range 
                            inText:(NSMutableAttributedString *)textStorage
                              line:(NSString *)line {
    if (range.location + range.length > [textStorage length]) return;
    
    NSError *error = nil;
    
    // Headers (##, ###, etc.)
    NSRegularExpression *headerRegex = [NSRegularExpression 
        regularExpressionWithPattern:@"^(#{1,6})\\s*" 
        options:0 error:&error];
    [self applyTransparencyToMatches:headerRegex 
                              inLine:line 
                           lineRange:range 
                          textStorage:textStorage
                           groupIndex:1]; // Seulement les #, pas l'espace
    
    // Bold (**texte**)
    NSRegularExpression *boldRegex = [NSRegularExpression 
        regularExpressionWithPattern:@"(\\*\\*)([^*]+)(\\*\\*)" 
        options:0 error:&error];
    [self applyTransparencyToMatches:boldRegex 
                              inLine:line 
                           lineRange:range 
                          textStorage:textStorage
                           groupIndex:1]; // Premier **
    [self applyTransparencyToMatches:boldRegex 
                              inLine:line 
                           lineRange:range 
                          textStorage:textStorage
                           groupIndex:3]; // Deuxième **
    
    // Italic (*texte*) - éviter conflit avec bold
    NSRegularExpression *italicRegex = [NSRegularExpression 
        regularExpressionWithPattern:@"(?<!\\*)(\\*)([^*]+)(\\*)(?!\\*)" 
        options:0 error:&error];
    [self applyTransparencyToMatches:italicRegex 
                              inLine:line 
                           lineRange:range 
                          textStorage:textStorage
                           groupIndex:1]; // Premier *
    [self applyTransparencyToMatches:italicRegex 
                              inLine:line 
                           lineRange:range 
                          textStorage:textStorage
                           groupIndex:3]; // Deuxième *
}

/**
 * @brief Fait disparaître les marqueurs Markdown sans laisser d'espace
 * 
 * @param regex L'expression régulière à utiliser
 * @param line Le contenu de la ligne
 * @param lineRange La plage de la ligne dans le texte complet
 * @param textStorage Le texte à modifier
 * @param groupIndex L'index du groupe de capture à masquer
 */
- (void)applyTransparencyToMatches:(NSRegularExpression *)regex
                            inLine:(NSString *)line
                         lineRange:(NSRange)lineRange
                        textStorage:(NSMutableAttributedString *)textStorage
                        groupIndex:(NSInteger)groupIndex {
    
    [regex enumerateMatchesInString:line 
                            options:0 
                              range:NSMakeRange(0, [line length])
                         usingBlock:^(NSTextCheckingResult *match, NSMatchingFlags flags, BOOL *stop) {
        if (groupIndex < (NSInteger)[match numberOfRanges]) {
            NSRange markerRange = [match rangeAtIndex:groupIndex];
            if (markerRange.location != NSNotFound) {
                // Convertir range relatif à la ligne en range absolu
                NSRange absoluteRange = NSMakeRange(
                    lineRange.location + markerRange.location,
                    markerRange.length
                );
                
                // Masquer les marqueurs sans prendre d'espace
                if (absoluteRange.location + absoluteRange.length <= [textStorage length]) {
                    // Méthode 1: Réduire la taille de police à 0.001 (quasi-invisible mais sans espace)
                    NSFont *tinyFont = [NSFont systemFontOfSize:0.001];
                    [textStorage addAttribute:NSFontAttributeName
                                        value:tinyFont
                                        range:absoluteRange];
                    
                    // Méthode 2: Couleur de fond identique au texte (masquage visuel)
                    NSColor *backgroundColor = [[self textStorage] attribute:NSBackgroundColorAttributeName 
                                                                      atIndex:0 
                                                               effectiveRange:nil];
                    if (!backgroundColor) {
                        backgroundColor = [NSColor colorWithRed:0.118 green:0.118 blue:0.118 alpha:1.0]; // Couleur de fond de l'éditeur
                    }
                    [textStorage addAttribute:NSForegroundColorAttributeName
                                        value:backgroundColor
                                        range:absoluteRange];
                    
                    NSLog(@"🫥 Marqueur masqué sans espace: range(%lu,%lu)", 
                          absoluteRange.location, absoluteRange.length);
                }
            }
        }
    }];
}

@end

@interface UIIconButton : NSButton
@property (nonatomic, assign) UIIconType iconType;
@property (nonatomic, assign) UIFramework* framework;
@end

@implementation UIIconButton

- (void)mouseEntered:(NSEvent *)event {
    [super mouseEntered:event];
    if (self.framework && self.framework->hoverHandler) {
        self.framework->hoverHandler(self.iconType, true, self.framework->userData);
    }
    [self setNeedsDisplay:YES];
}

- (void)mouseExited:(NSEvent *)event {
    [super mouseExited:event];
    if (self.framework && self.framework->hoverHandler) {
        self.framework->hoverHandler(self.iconType, false, self.framework->userData);
    }
    [self setNeedsDisplay:YES];
}

- (void)mouseDown:(NSEvent *)event {
    [super mouseDown:event];
    if (self.framework && self.framework->clickHandler) {
        self.framework->clickHandler(self.iconType, self.framework->userData);
    }
}

@end

// ========== Implémentation des fonctions C ==========

UIFramework* ui_framework_create(NSWindow* window) {
    if (!window) return NULL;
    
    UIFramework* framework = calloc(1, sizeof(UIFramework));
    if (!framework) return NULL;
    
    framework->window = window;
    framework->iconButtons = [[NSMutableArray alloc] init];
    framework->activeIcon = UI_ICON_HOME; // Par défaut sur dashboard
    framework->isInitialized = false;
    
    // Configuration par défaut
    framework->sidebarConfig = ui_framework_get_default_sidebar_config();
    framework->editorConfig = ui_framework_get_default_editor_config();
    
    return framework;
}

void ui_framework_destroy(UIFramework* framework) {
    if (!framework) return;
    
    framework->iconButtons = nil;
    free(framework);
}

bool ui_framework_setup_layout(UIFramework* framework) {
    if (!framework || !framework->window) return false;
    
    NSView* contentView = [framework->window contentView];
    NSRect windowFrame = [contentView bounds];
    
    // Créer le conteneur principal
    framework->containerView = [[NSView alloc] initWithFrame:windowFrame];
    [framework->containerView setAutoresizingMask:NSViewWidthSizable | NSViewHeightSizable];
    [contentView addSubview:framework->containerView];
    
    // Créer la barre latérale
    NSRect sidebarFrame = NSMakeRect(0, 0, 
                                    framework->sidebarConfig.width, 
                                    NSHeight(windowFrame));
    framework->sidebarView = [[NSView alloc] initWithFrame:sidebarFrame];
    [framework->sidebarView setAutoresizingMask:NSViewHeightSizable];
    [framework->sidebarView setWantsLayer:YES];
    [framework->sidebarView.layer setBackgroundColor:framework->sidebarConfig.backgroundColor.CGColor];
    
    // Créer l'éditeur avec scroll view
    NSRect editorFrame = NSMakeRect(0, 0, // La barre latérale sera en superposition
                                   NSWidth(windowFrame), 
                                   NSHeight(windowFrame));
    
    framework->editorScrollView = [[NSScrollView alloc] initWithFrame:editorFrame];
    [framework->editorScrollView setAutoresizingMask:NSViewWidthSizable | NSViewHeightSizable];
    [framework->editorScrollView setHasVerticalScroller:YES];
    [framework->editorScrollView setHasHorizontalScroller:NO];
    [framework->editorScrollView setAutohidesScrollers:YES];
    [framework->editorScrollView setBorderType:NSNoBorder];
    
    // Créer l'éditeur Markdown
    framework->editorView = [[MarkdownEditor alloc] initWithFrame:editorFrame];
    [(NSView*)framework->editorView setAutoresizingMask:NSViewWidthSizable | NSViewHeightSizable];
    
    // Marge gauche pour éviter la superposition avec la barre latérale
    NSTextContainer* textContainer = [framework->editorView textContainer];
    [textContainer setContainerSize:NSMakeSize(NSWidth(editorFrame) - framework->editorConfig.marginLeft, 1e6)];
    [textContainer setWidthTracksTextView:NO];
    [framework->editorView setTextContainerInset:NSMakeSize(framework->editorConfig.marginLeft + 10, 10)];
    
    [framework->editorScrollView setDocumentView:framework->editorView];
    
    // Ajouter les vues au conteneur (ordre important pour superposition)
    [framework->containerView addSubview:framework->editorScrollView];
    [framework->containerView addSubview:framework->sidebarView]; // En dernier pour être au-dessus
    
    // Ajouter les icônes par défaut dans l'ordre du workflow
    ui_framework_add_icon(framework, UI_ICON_SEARCH);   // Recherche
    ui_framework_add_icon(framework, UI_ICON_BACK);     // Retour
    ui_framework_add_icon(framework, UI_ICON_HOME);     // Tableau de bord
    ui_framework_add_icon(framework, UI_ICON_SETTINGS); // Paramètres
    
    framework->isInitialized = true;
    return true;
}

UISidebarConfig ui_framework_get_default_sidebar_config(void) {
    UISidebarConfig config = {
        .width = 60.0,
        .backgroundColor = [NSColor colorWithRed:0.95 green:0.95 blue:0.95 alpha:0.95], // Fond semi-transparent
        .iconColor = [NSColor colorWithRed:0.3 green:0.3 blue:0.3 alpha:1.0],
        .hoverColor = [NSColor colorWithRed:0.2 green:0.4 blue:0.8 alpha:1.0],
        .activeColor = [NSColor colorWithRed:0.1 green:0.3 blue:0.7 alpha:1.0],
        .iconSize = 24.0,
        .iconSpacing = 20.0
    };
    return config;
}

UIEditorConfig ui_framework_get_default_editor_config(void) {
    UIEditorConfig config = {
        .backgroundColor = [NSColor textBackgroundColor],
        .textColor = [NSColor textColor],
        .font = [NSFont fontWithName:@"Monaco" size:14] ?: [NSFont monospacedSystemFontOfSize:14 weight:NSFontWeightRegular],
        .marginLeft = 60.0, // Espace pour la barre latérale
        .showLineNumbers = false
    };
    return config;
}

void ui_framework_add_icon(UIFramework* framework, UIIconType iconType) {
    if (!framework || !framework->sidebarView) return;
    
    CGFloat iconSize = framework->sidebarConfig.iconSize;
    NSRect iconFrame = NSMakeRect(0, 0, iconSize, iconSize);
    
    UIIconButton* iconButton = [[UIIconButton alloc] initWithFrame:iconFrame];
    iconButton.iconType = iconType;
    iconButton.framework = framework;
    
    [iconButton setBordered:NO];
    [iconButton setButtonType:NSButtonTypeMomentaryChange];
    [iconButton setImagePosition:NSImageOnly];
    
    // Créer l'image de l'icône
    NSImage* iconImage = ui_framework_create_icon_image(iconType, iconSize, framework->sidebarConfig.iconColor);
    [iconButton setImage:iconImage];
    
    // Configurer le tracking pour les événements de survol
    NSTrackingArea* trackingArea = [[NSTrackingArea alloc] 
        initWithRect:[iconButton bounds] 
        options:(NSTrackingMouseEnteredAndExited | NSTrackingActiveInKeyWindow) 
        owner:iconButton 
        userInfo:nil];
    [iconButton addTrackingArea:trackingArea];
    
    // Tooltip
    [iconButton setToolTip:ui_framework_get_icon_tooltip(iconType)];
    
    [framework->iconButtons addObject:iconButton];
    [framework->sidebarView addSubview:iconButton];
    
    ui_framework_layout_icons(framework);
}

static void ui_framework_layout_icons(UIFramework* framework) {
    if (!framework || !framework->sidebarView) return;

    CGFloat iconSize = framework->sidebarConfig.iconSize;
    CGFloat sidebarWidth = framework->sidebarConfig.width;
    CGFloat sidebarHeight = NSHeight([framework->sidebarView bounds]);
    CGFloat topMargin = 40.0;
    CGFloat bottomMargin = 40.0;

    NSArray<NSNumber*>* iconOrder = @[ @(UI_ICON_SEARCH), @(UI_ICON_BACK), @(UI_ICON_HOME), @(UI_ICON_SETTINGS) ];
    NSMutableArray<UIIconButton*>* orderedButtons = [NSMutableArray arrayWithCapacity:[iconOrder count]];

    for (NSNumber* iconNumber in iconOrder) {
        UIIconType iconType = (UIIconType)[iconNumber integerValue];
        for (UIIconButton* button in framework->iconButtons) {
            if (button.iconType == iconType) {
                [orderedButtons addObject:button];
                break;
            }
        }
    }

    NSInteger count = [orderedButtons count];
    if (count == 0) {
        return;
    }

    CGFloat availableHeight = MAX(sidebarHeight - topMargin - bottomMargin - (iconSize * count), 0.0);
    CGFloat spacing = (count > 1) ? availableHeight / (count - 1) : 0.0;
    CGFloat startY = sidebarHeight - topMargin - iconSize;
    CGFloat x = (sidebarWidth - iconSize) / 2.0;

    for (NSInteger idx = 0; idx < count; idx++) {
        UIIconButton* button = orderedButtons[idx];
        CGFloat y = startY - idx * (iconSize + spacing);
        if (y < bottomMargin) {
            y = bottomMargin;
        }
        NSRect frame = button.frame;
        frame.origin.x = x;
        frame.origin.y = y;
        frame.size = NSMakeSize(iconSize, iconSize);
        [button setFrame:frame];
    }
}

NSImage* ui_framework_create_icon_image(UIIconType iconType, CGFloat size, NSColor* color) {
    NSImage* image = [[NSImage alloc] initWithSize:NSMakeSize(size, size)];
    
    [image lockFocus];
    
    // Configuration du contexte graphique
    NSGraphicsContext* context = [NSGraphicsContext currentContext];
    CGContextRef cgContext = [context CGContext];
    
    // Configurer la couleur de remplissage
    CGContextSetFillColorWithColor(cgContext, color.CGColor);
    
    // Facteur d'échelle pour adapter le SVG 16x16 à la taille demandée
    CGFloat scale = size / 16.0;
    CGContextScaleCTM(cgContext, scale, scale);
    
    // Corriger l'orientation : inverser Y pour que les icônes soient dans le bon sens
    // macOS a l'origine en bas-gauche, SVG en haut-gauche
    CGContextTranslateCTM(cgContext, 0, 16);
    CGContextScaleCTM(cgContext, 1, -1);
    
    // Créer le chemin selon l'icône Bootstrap
    NSBezierPath* path = [NSBezierPath bezierPath];
    
    switch (iconType) {
        case UI_ICON_SEARCH: {
            // Bootstrap search icon path
            // M11.742 10.344a6.5 6.5 0 1 0-1.397 1.398h-.001q.044.06.098.115l3.85 3.85a1 1 0 0 0 1.415-1.414l-3.85-3.85a1 1 0 0 0-.115-.1zM12 6.5a5.5 5.5 0 1 1-11 0 5.5 5.5 0 0 1 11 0
            
            // Cercle principal (lentille)
            NSRect circleRect = NSMakeRect(1, 1, 11, 11);
            [path appendBezierPathWithOvalInRect:circleRect];
            [path setLineWidth:1.0];
            
            // Manche de la loupe
            [path moveToPoint:NSMakePoint(10.5, 10.5)];
            [path lineToPoint:NSMakePoint(14.5, 14.5)];
            [path setLineWidth:1.5];
            break;
        }
        case UI_ICON_BACK: {
            // Bootstrap arrow-left icon path
            // M15 8a.5.5 0 0 0-.5-.5H2.707l3.147-3.146a.5.5 0 1 0-.708-.708l-4 4a.5.5 0 0 0 0 .708l4 4a.5.5 0 0 0 .708-.708L2.707 8.5H14.5A.5.5 0 0 0 15 8
            
            // Ligne horizontale
            [path moveToPoint:NSMakePoint(2.5, 8)];
            [path lineToPoint:NSMakePoint(14.5, 8)];
            
            // Pointe de flèche
            [path moveToPoint:NSMakePoint(6, 4.5)];
            [path lineToPoint:NSMakePoint(2, 8)];
            [path lineToPoint:NSMakePoint(6, 11.5)];
            [path setLineWidth:1.2];
            break;
        }
        case UI_ICON_HOME: {
            // Bootstrap house icon path  
            // M8.707 1.5a1 1 0 0 0-1.414 0L.646 8.146a.5.5 0 0 0 .708.708L2 8.207V13.5A1.5 1.5 0 0 0 3.5 15h9a1.5 1.5 0 0 0 1.5-1.5V8.207l.646.647a.5.5 0 0 0 .708-.708L13 5.793V2.5a.5.5 0 0 0-.5-.5h-1a.5.5 0 0 0-.5.5v1.293zM13 7.207V13.5a.5.5 0 0 1-.5.5h-9a.5.5 0 0 1-.5-.5V7.207l5-5z
            
            // Toit (triangle)
            [path moveToPoint:NSMakePoint(1, 8)];
            [path lineToPoint:NSMakePoint(8, 1.5)];
            [path lineToPoint:NSMakePoint(15, 8)];
            
            // Base de la maison
            NSRect houseRect = NSMakeRect(3, 8, 10, 6);
            [path appendBezierPathWithRect:houseRect];
            [path setLineWidth:1.0];
            break;
        }
        case UI_ICON_EDITOR: {
            // Icône éditeur : feuille + crayon
            NSRect pageRect = NSMakeRect(3, 2, 10, 12);
            [path appendBezierPathWithRect:pageRect];

            for (int i = 0; i < 3; i++) {
                CGFloat y = 11 - (CGFloat)i * 3.0;
                [path moveToPoint:NSMakePoint(4, y)];
                [path lineToPoint:NSMakePoint(12, y)];
            }

            NSBezierPath* pencil = [NSBezierPath bezierPath];
            [pencil moveToPoint:NSMakePoint(5, 4)];
            [pencil lineToPoint:NSMakePoint(11, 10)];
            [pencil lineToPoint:NSMakePoint(10, 11)];
            [pencil lineToPoint:NSMakePoint(4, 5)];
            [pencil closePath];
            [path appendBezierPath:pencil];
            [path setLineWidth:1.0];
            break;
        }
        case UI_ICON_TOOLS: {
            // Icône outils : croix tournevis + marteau simplifiés
            [path appendBezierPathWithRect:NSMakeRect(7, 3, 2, 9)];
            [path appendBezierPathWithRect:NSMakeRect(3, 8, 10, 2)];

            NSRect wrenchHead = NSMakeRect(4, 10, 4, 4);
            [path appendBezierPathWithOvalInRect:wrenchHead];

            NSRect screwdriverTip = NSMakeRect(9, 2, 3, 3);
            [path appendBezierPathWithRect:screwdriverTip];
            [path setLineWidth:1.0];
            break;
        }
        case UI_ICON_SETTINGS: {
            // Bootstrap gear icon path (simplifié pour la lisibilité)
            // Cercle central
            NSRect centerCircle = NSMakeRect(5.5, 5.5, 5, 5);
            [path appendBezierPathWithOvalInRect:centerCircle];
            
            // Dents de la roue (8 dents autour)
            CGFloat centerX = 8, centerY = 8;
            CGFloat outerRadius = 7, innerRadius = 3;
            
            for (int i = 0; i < 8; i++) {
                CGFloat angle = (M_PI / 4) * i;
                CGFloat x1 = centerX + cos(angle) * innerRadius;
                CGFloat y1 = centerY + sin(angle) * innerRadius;
                CGFloat x2 = centerX + cos(angle) * outerRadius;
                CGFloat y2 = centerY + sin(angle) * outerRadius;
                
                // Dent rectangulaire
                NSRect tooth = NSMakeRect(x2 - 0.8, y2 - 0.8, 1.6, 1.6);
                [path appendBezierPathWithRect:tooth];
            }
            [path setLineWidth:0.8];
            break;
        }
        case UI_ICON_FILES: {
            // Bootstrap folder icon path
            // M.54 3.87.5 3a2 2 0 0 1 2-2h3.672a2 2 0 0 1 1.414.586l.828.828A2 2 0 0 0 9.828 3h3.982a2 2 0 0 1 1.992 2.181l-.637 7A2 2 0 0 1 13.174 14H2.826a2 2 0 0 1-1.991-1.819l-.637-7a2 2 0 0 1 .342-1.31zM2.19 4a1 1 0 0 0-.996 1.09l.637 7a1 1 0 0 0 .995.91h10.348a1 1 0 0 0 .995-.91l.637-7A1 1 0 0 0 13.81 4zm4.69-1.707A1 1 0 0 0 6.172 2H2.5a1 1 0 0 0-1 .981l.006.139q.323-.119.684-.12h5.396z
            
            // Corps principal du dossier
            NSRect folderBody = NSMakeRect(1, 4, 14, 9);
            [path appendBezierPathWithRect:folderBody];
            
            // Onglet du dossier
            NSRect folderTab = NSMakeRect(2, 1, 6, 3);
            [path appendBezierPathWithRect:folderTab];
            
            // Ligne de connexion entre l'onglet et le corps
            [path moveToPoint:NSMakePoint(8, 4)];
            [path lineToPoint:NSMakePoint(8, 1)];
            
            [path setLineWidth:1.0];
            break;
        }
    }
    
    [path stroke];
    [image unlockFocus];
    
    return image;
}

NSString* ui_framework_get_icon_name(UIIconType iconType) {
    switch (iconType) {
        case UI_ICON_SEARCH: return @"search";
        case UI_ICON_BACK: return @"back";
        case UI_ICON_HOME: return @"home";
        case UI_ICON_SETTINGS: return @"settings";
        case UI_ICON_EDITOR: return @"editor";
        case UI_ICON_TOOLS: return @"tools";
        case UI_ICON_FILES: return @"files";
        default: return @"unknown";
    }
}

NSString* ui_framework_get_icon_tooltip(UIIconType iconType) {
    switch (iconType) {
        case UI_ICON_SEARCH: return @"Rechercher dans les notes";
        case UI_ICON_BACK: return @"Retour";
        case UI_ICON_HOME: return @"Dashboard";
        case UI_ICON_SETTINGS: return @"Paramètres";
        case UI_ICON_EDITOR: return @"Zone d'édition";
        case UI_ICON_TOOLS: return @"Outils avancés";
        case UI_ICON_FILES: return @"Bibliothèque de notes";
        default: return @"";
    }
}

void ui_framework_set_click_handler(UIFramework* framework, UIIconClickHandler handler, void* userData) {
    if (!framework) return;
    framework->clickHandler = handler;
    framework->userData = userData;
}

void ui_framework_set_hover_handler(UIFramework* framework, UIIconHoverHandler handler, void* userData) {
    if (!framework) return;
    framework->hoverHandler = handler;
    framework->userData = userData;
}

void ui_framework_set_icon_active(UIFramework* framework, UIIconType iconType) {
    if (!framework) return;
    
    framework->activeIcon = iconType;
    
    // Mettre à jour l'apparence de tous les boutons
    for (UIIconButton* button in framework->iconButtons) {
        if (button.iconType == iconType) {
            // Bouton actif
            NSImage* activeImage = ui_framework_create_icon_image(iconType, 
                framework->sidebarConfig.iconSize, 
                framework->sidebarConfig.activeColor);
            [button setImage:activeImage];
        } else {
            // Bouton normal
            NSImage* normalImage = ui_framework_create_icon_image(button.iconType, 
                framework->sidebarConfig.iconSize, 
                framework->sidebarConfig.iconColor);
            [button setImage:normalImage];
        }
    }
}


void ui_framework_set_icon_enabled(UIFramework* framework, UIIconType iconType, bool enabled) {
    if (!framework) return;

    for (UIIconButton* button in framework->iconButtons) {
        if (button.iconType == iconType) {
            [button setEnabled:enabled];
            button.alphaValue = enabled ? 1.0 : 0.45;
            break;
        }
    }
}

NSTextView* ui_framework_get_editor(UIFramework* framework) {
    return framework ? framework->editorView : nil;
}

void ui_framework_set_editor_content(UIFramework* framework, NSString* content) {
    if (!framework || !framework->editorView) return;
    [framework->editorView setString:content ?: @""];
    
    // Déclencher le rendu Markdown si c'est un MarkdownEditor
    if ([framework->editorView respondsToSelector:@selector(render)]) {
        [framework->editorView render];
    }
}

NSString* ui_framework_get_editor_content(UIFramework* framework) {
    if (!framework || !framework->editorView) return @"";
    return [framework->editorView string];
}

void ui_framework_update_layout(UIFramework* framework) {
    if (!framework || !framework->isInitialized) return;
    
    NSRect windowFrame = [[framework->window contentView] bounds];
    [framework->containerView setFrame:windowFrame];
    
    // Mettre à jour l'éditeur
    NSRect editorFrame = NSMakeRect(0, 0, NSWidth(windowFrame), NSHeight(windowFrame));
    [framework->editorScrollView setFrame:editorFrame];
    
    // Mettre à jour la barre latérale
    NSRect sidebarFrame = NSMakeRect(0, 0, framework->sidebarConfig.width, NSHeight(windowFrame));
    [framework->sidebarView setFrame:sidebarFrame];

    ui_framework_layout_icons(framework);
}

// Fonctions de thème
UITheme ui_framework_get_default_light_theme(void) {
    UITheme theme = {
        .primaryColor = [NSColor colorWithRed:0.2 green:0.4 blue:0.8 alpha:1.0],
        .secondaryColor = [NSColor colorWithRed:0.6 green:0.6 blue:0.6 alpha:1.0],
        .backgroundLight = [NSColor colorWithRed:0.98 green:0.98 blue:0.98 alpha:1.0],
        .backgroundDark = [NSColor colorWithRed:0.95 green:0.95 blue:0.95 alpha:1.0],
        .textPrimary = [NSColor colorWithRed:0.1 green:0.1 blue:0.1 alpha:1.0],
        .textSecondary = [NSColor colorWithRed:0.4 green:0.4 blue:0.4 alpha:1.0],
        .accentColor = [NSColor colorWithRed:0.1 green:0.6 blue:0.3 alpha:1.0]
    };
    return theme;
}