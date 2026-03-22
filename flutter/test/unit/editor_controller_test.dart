import 'package:flutter_test/flutter_test.dart';
import 'package:flutter/services.dart';
import 'package:c_editor_flutter/core/editor/editor_controller.dart';
import 'package:c_editor_flutter/models/models.dart';

void main() {
  group('EditorController', () {
    late EditorController controller;

    setUp(() {
      controller = EditorController();
    });

    tearDown(() {
      controller.dispose();
    });

    group('Basic Operations', () {
      test('should initialize with empty state', () {
        expect(controller.state.text, isEmpty);
        expect(controller.state.selection, const TextSelection.collapsed(offset: 0));
        expect(controller.state.isDirty, false);
        expect(controller.canUndo, false);
        expect(controller.canRedo, false);
      });

      test('should set content correctly', () {
        const content = '# Hello World\n\nThis is a test.';
        controller.setContent(content);

        expect(controller.state.text, content);
        expect(controller.state.isDirty, false);
        expect(controller.state.undoStack, isEmpty);
      });

      test('should insert text at cursor position', () {
        controller.setContent('Hello world');
        controller.updateSelection(const TextSelection.collapsed(offset: 5));
        
        controller.insertText(' beautiful');
        
        expect(controller.state.text, 'Hello beautiful world');
        expect(controller.state.selection.start, 15); // After inserted text
        expect(controller.state.isDirty, true);
      });

      test('should replace selected text', () {
        controller.setContent('Hello world');
        controller.updateSelection(const TextSelection(baseOffset: 6, extentOffset: 11));
        
        controller.insertText('Flutter');
        
        expect(controller.state.text, 'Hello Flutter');
        expect(controller.state.selection.start, 13);
      });
    });

    group('Formatting Operations', () {
      test('should apply bold formatting around selection', () {
        controller.setContent('Hello world');
        controller.updateSelection(const TextSelection(baseOffset: 6, extentOffset: 11));
        
        controller.insertFormatting('**', '**');
        
        expect(controller.state.text, 'Hello **world**');
        expect(controller.state.selection.baseOffset, 8);
        expect(controller.state.selection.extentOffset, 13);
      });

      test('should insert formatting at cursor when no selection', () {
        controller.setContent('Hello world');
        controller.updateSelection(const TextSelection.collapsed(offset: 6));
        
        controller.insertFormatting('**', '**');
        
        expect(controller.state.text, 'Hello ****world');
        expect(controller.state.selection.start, 8);
      });

      test('should handle complex formatting combinations', () {
        controller.setContent('Some text here');
        controller.updateSelection(const TextSelection(baseOffset: 5, extentOffset: 9));
        
        // Apply bold
        controller.insertFormatting('**', '**');
        expect(controller.state.text, 'Some **text** here');
        
        // Select bold text and apply italic
        controller.updateSelection(const TextSelection(baseOffset: 7, extentOffset: 11));
        controller.insertFormatting('*', '*');
        expect(controller.state.text, 'Some ***text*** here');
      });
    });

    group('Undo/Redo System', () {
      test('should create undo point on text insertion', () {
        controller.setContent('Initial text');
        expect(controller.canUndo, false);
        
        controller.insertText(' added');
        expect(controller.canUndo, true);
        expect(controller.canRedo, false);
        
        controller.undo();
        expect(controller.state.text, 'Initial text');
        expect(controller.canUndo, false);
        expect(controller.canRedo, true);
      });

      test('should handle multiple undo/redo operations', () {
        controller.setContent('Start');
        
        // Make several changes
        controller.insertText(' step1');
        controller.insertText(' step2');
        controller.insertText(' step3');
        
        expect(controller.state.text, 'Start step1 step2 step3');
        
        // Undo all changes
        controller.undo();
        expect(controller.state.text, 'Start step1 step2');
        
        controller.undo();
        expect(controller.state.text, 'Start step1');
        
        controller.undo();
        expect(controller.state.text, 'Start');
        
        // Redo changes
        controller.redo();
        expect(controller.state.text, 'Start step1');
        
        controller.redo();
        expect(controller.state.text, 'Start step1 step2');
      });

      test('should limit undo stack size', () {
        controller.setContent('Start');
        
        // Make more than max undo steps
        for (int i = 0; i < EditorController.maxUndoSteps + 10; i++) {
          controller.insertText(' $i');
        }
        
        expect(controller.state.undoStack.length, lessThanOrEqualTo(EditorController.maxUndoSteps));
      });

      test('should clear redo stack on new action after undo', () {
        controller.setContent('Start');
        controller.insertText(' step1');
        controller.insertText(' step2');
        
        // Undo once
        controller.undo();
        expect(controller.canRedo, true);
        
        // Make new change - should clear redo stack
        controller.insertText(' new');
        expect(controller.canRedo, false);
        expect(controller.state.text, 'Start step1 new');
      });
    });

    group('Text Manipulation', () {
      test('should replace text in range correctly', () {
        controller.setContent('Hello beautiful world');
        
        controller.replaceText(6, 15, 'amazing');
        
        expect(controller.state.text, 'Hello amazing world');
        expect(controller.state.selection.start, 13); // After replacement
      });

      test('should handle invalid range gracefully', () {
        controller.setContent('Hello world');
        
        // Invalid ranges should be ignored
        controller.replaceText(-1, 5, 'test'); // negative start
        expect(controller.state.text, 'Hello world');
        
        controller.replaceText(5, 100, 'test'); // end beyond text
        expect(controller.state.text, 'Hello world');
        
        controller.replaceText(8, 5, 'test'); // start > end
        expect(controller.state.text, 'Hello world');
      });

      test('should get text range correctly', () {
        controller.setContent('Hello world test');
        
        expect(controller.getTextRange(0, 5), 'Hello');
        expect(controller.getTextRange(6, 11), 'world');
        expect(controller.getTextRange(12, 16), 'test');
        
        // Invalid ranges should return empty string
        expect(controller.getTextRange(-1, 5), '');
        expect(controller.getTextRange(5, 100), '');
        expect(controller.getTextRange(8, 5), '');
      });
    });

    group('Line Operations', () {
      test('should get current line correctly', () {
        controller.setContent('Line 1\nLine 2\nLine 3');
        
        // Position at start of line 2
        controller.updateSelection(const TextSelection.collapsed(offset: 7));
        expect(controller.getCurrentLine(), 'Line 2');
        
        // Position at end of line 1
        controller.updateSelection(const TextSelection.collapsed(offset: 6));
        expect(controller.getCurrentLine(), 'Line 1');
      });

      test('should get current word correctly', () {
        controller.setContent('Hello beautiful world');
        
        // Position in middle of "beautiful"
        controller.updateSelection(const TextSelection.collapsed(offset: 8));
        expect(controller.getCurrentWord(), 'beautiful');
        
        // Position at word boundary
        controller.updateSelection(const TextSelection.collapsed(offset: 5));
        expect(controller.getCurrentWord(), '');
      });

      test('should duplicate line correctly', () {
        controller.setContent('Line 1\nLine 2\nLine 3');
        
        // Position in line 2
        controller.updateSelection(const TextSelection.collapsed(offset: 9));
        controller.duplicateLine();
        
        expect(controller.state.text, 'Line 1\nLine 2\nLine 2\nLine 3');
        expect(controller.canUndo, true);
      });
    });

    group('Selection Management', () {
      test('should update selection correctly', () {
        controller.setContent('Hello world');
        
        const selection = TextSelection(baseOffset: 2, extentOffset: 7);
        controller.updateSelection(selection);
        
        expect(controller.state.selection, selection);
      });

      test('should select all text', () {
        controller.setContent('Hello world');
        controller.selectAll();
        
        expect(controller.state.selection.baseOffset, 0);
        expect(controller.state.selection.extentOffset, 11);
      });

      test('should clear content', () {
        controller.setContent('Hello world');
        controller.clear();
        
        expect(controller.state.text, '');
        expect(controller.state.selection, const TextSelection.collapsed(offset: 0));
        expect(controller.canUndo, true);
      });
    });

    group('State Management', () {
      test('should mark as saved correctly', () {
        controller.setContent('Hello world');
        controller.insertText(' test');
        
        expect(controller.state.isDirty, true);
        
        controller.markSaved();
        expect(controller.state.isDirty, false);
      });

      test('should track dirty state correctly', () {
        controller.setContent('Hello');
        expect(controller.state.isDirty, false);
        
        controller.insertText(' world');
        expect(controller.state.isDirty, true);
        
        controller.undo();
        expect(controller.state.isDirty, false);
      });
    });
  });
}