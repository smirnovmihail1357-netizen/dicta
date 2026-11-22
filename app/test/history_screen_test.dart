// HIST-02/HIST-03: journal browsing, substring search and the three row
// actions. Navigation taps the rail before asserts — offstage tabs are hidden
// from find.text by default (skipOffstage).
import 'dart:async';

import 'package:dicta_app/core/engine.dart';
import 'package:dicta_app/features/history/history.dart';
import 'package:dicta_app/features/history/history_screen.dart';
import 'package:dicta_app/features/shell/shell.dart';
import 'package:flutter/material.dart';
import 'package:flutter/semantics.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:intl/intl.dart';

import 'fake_dicta_engine.dart';

Future<void> tick(WidgetTester tester) async {
  await tester.pump(const Duration(milliseconds: 100));
  await tester.pump();
}

Future<void> captureTextViaStop(WidgetTester tester) async {
  await tester.tap(find.byKey(const Key('dashStopButton')));
  await tester.pump();
  await tester.pump();
}

Future<void> goToHistory(WidgetTester tester) async {
  await tester.tap(find.text('История'));
  await tick(tester);
}

// Shared base of ElevatedButton / FilledButton / OutlinedButton: the row's
// priority matrix mixes all three, so the gate helper must not assume one.
bool rowButtonEnabled(WidgetTester tester, String key) =>
    tester.widget<ButtonStyleButton>(find.byKey(Key(key))).onPressed != null;

void main() {
  testWidgets('captured dictation is listed with an HH:mm timestamp',
      (tester) async {
    final fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.recording),
      stopReply: const DictaStatus(
        connected: true,
        state: DaemonState.idle,
        text: 'привет',
      ),
    );
    ProviderContainer? container;
    await tester.pumpWidget(
      buildFakeApp(fake, onCopy: (_) {}, captureContainer: (c) => container = c),
    );
    await tick(tester);
    await captureTextViaStop(tester);
    await goToHistory(tester);

    final at = container!.read(historyProvider).first.at;
    expect(find.text('привет'), findsOneWidget);
    expect(find.text(DateFormat.Hm('ru').format(at)), findsOneWidget);

    await tester.pumpWidget(const SizedBox.shrink());
  });

  testWidgets('search is case-insensitive substring, no debounce, empty = all',
      (tester) async {
    final fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.recording),
      stopReply: const DictaStatus(
        connected: true,
        state: DaemonState.idle,
        text: 'мир',
      ),
    );
    await tester.pumpWidget(buildFakeApp(fake, onCopy: (_) {}));
    await tick(tester);
    await captureTextViaStop(tester);
    fake.stopReply = const DictaStatus(
      connected: true,
      state: DaemonState.idle,
      text: 'солнце',
    );
    await captureTextViaStop(tester);
    await goToHistory(tester);

    expect(find.text('мир'), findsOneWidget);
    expect(find.text('солнце'), findsOneWidget);

    await tester.enterText(find.byKey(const Key('historySearchField')), 'МИР');
    await tester.pump();
    expect(find.text('мир'), findsOneWidget);
    expect(find.text('солнце'), findsNothing);

    await tester.enterText(find.byKey(const Key('historySearchField')), 'луна');
    await tester.pump();
    expect(find.text('ничего не найдено'), findsOneWidget);

    await tester.enterText(find.byKey(const Key('historySearchField')), '');
    await tester.pump();
    expect(find.text('мир'), findsOneWidget);
    expect(find.text('солнце'), findsOneWidget);

    await tester.pumpWidget(const SizedBox.shrink());
  });

  testWidgets('copy uses the seam on explicit click and reports locally',
      (tester) async {
    final fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.recording),
      stopReply: const DictaStatus(
        connected: true,
        state: DaemonState.idle,
        text: 'привет',
      ),
    );
    final copied = <String>[];
    await tester.pumpWidget(buildFakeApp(fake, onCopy: copied.add));
    await tick(tester);
    await captureTextViaStop(tester);
    await goToHistory(tester);

    await tester.tap(find.byKey(const Key('histCopyButton-1')));
    await tester.pump();
    await tester.pump();

    expect(copied, <String>['привет']);
    expect(find.text('скопировано'), findsOneWidget);

    await tester.pumpWidget(const SizedBox.shrink());
  });

  testWidgets('copy snackbar is exposed to assistive technology (AT probe)',
      (tester) async {
    final fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.recording),
      stopReply: const DictaStatus(
        connected: true,
        state: DaemonState.idle,
        text: 'привет',
      ),
    );
    final handle = tester.ensureSemantics();
    try {
      await tester.pumpWidget(buildFakeApp(fake, onCopy: (_) {}));
      await tick(tester);
      await captureTextViaStop(tester);
      await goToHistory(tester);

      await tester.tap(find.byKey(const Key('histCopyButton-1')));
      await tester.pump();
      await tester.pump();

      expect(find.bySemanticsLabel('скопировано'), findsOneWidget);
    } finally {
      handle.dispose();
    }

    await tester.pumpWidget(const SizedBox.shrink());
  });

  testWidgets('re-insert goes through inject and is gated while in flight',
      (tester) async {
    final gate = Completer<void>();
    final fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.recording),
      stopReply: const DictaStatus(
        connected: true,
        state: DaemonState.idle,
        text: 'привет',
      ),
      injectGate: gate,
    );
    ProviderContainer? container;
    await tester.pumpWidget(buildFakeApp(
      fake,
      onCopy: (_) {},
      captureContainer: (c) => container = c,
    ));
    await tick(tester);
    await captureTextViaStop(tester);
    await goToHistory(tester);

    expect(rowButtonEnabled(tester, 'histInjectButton-1'), isTrue);
    await tester.tap(find.byKey(const Key('histInjectButton-1')));
    await tester.pump();
    expect(fake.injectTexts, <String>['привет']);
    expect(rowButtonEnabled(tester, 'histInjectButton-1'), isFalse);

    gate.complete();
    await tester.pump();
    await tester.pump();
    expect(find.text('текст вставлен'), findsOneWidget);
    // The re-insert path is inject, never a second capture: journal length is
    // unchanged by the echo (IN-01).
    expect(container!.read(historyProvider), hasLength(1));

    await tester.pumpWidget(const SizedBox.shrink());
  });

  testWidgets('delete removes the entry locally and reports it',
      (tester) async {
    final fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.recording),
      stopReply: const DictaStatus(
        connected: true,
        state: DaemonState.idle,
        text: 'привет',
      ),
    );
    ProviderContainer? container;
    await tester.pumpWidget(
      buildFakeApp(fake, onCopy: (_) {}, captureContainer: (c) => container = c),
    );
    await tick(tester);
    await captureTextViaStop(tester);
    await goToHistory(tester);

    await tester.tap(find.byKey(const Key('histDeleteButton-1')));
    await tester.pump();
    await tester.pump();

    expect(container!.read(historyProvider), isEmpty);
    expect(find.text('привет'), findsNothing);
    expect(find.text('удалено'), findsOneWidget);
    expect(find.text('журнал пуст'), findsOneWidget);

    await tester.pumpWidget(const SizedBox.shrink());
  });

  testWidgets('undo restores a deleted entry at its original index',
      (tester) async {
    final fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.recording),
      stopReply: const DictaStatus(
        connected: true,
        state: DaemonState.idle,
        text: 'первый',
      ),
    );
    ProviderContainer? container;
    await tester.pumpWidget(
      buildFakeApp(fake, onCopy: (_) {}, captureContainer: (c) => container = c),
    );
    await tick(tester);
    await captureTextViaStop(tester);
    fake.stopReply = const DictaStatus(
      connected: true,
      state: DaemonState.idle,
      text: 'второй',
    );
    await captureTextViaStop(tester);
    await goToHistory(tester);

    // Newest-first: ['второй', 'первый'].
    expect(
      container!.read(historyProvider).map((e) => e.text).toList(),
      <String>['второй', 'первый'],
    );

    // Delete the older entry (index 1); undo must put it back in that slot.
    await tester.tap(find.byKey(const Key('histDeleteButton-1')));
    await tester.pump();
    // Let the snackbar finish its slide-in so its action is hit-testable.
    await tester.pump(const Duration(milliseconds: 400));
    await tester.pump();
    expect(
      container!.read(historyProvider).map((e) => e.text).toList(),
      <String>['второй'],
    );
    expect(find.text('удалено'), findsOneWidget);
    expect(find.text('вернуть'), findsOneWidget);

    await tester.tap(find.text('вернуть'));
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 400));
    await tester.pump();
    expect(
      container!.read(historyProvider).map((e) => e.text).toList(),
      <String>['второй', 'первый'],
    );
    expect(find.text('первый'), findsOneWidget);

    await tester.pumpWidget(const SizedBox.shrink());
  });

  testWidgets('failed inject reports the daemon error in a snackbar',
      (tester) async {
    final fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.recording),
      stopReply: const DictaStatus(
        connected: true,
        state: DaemonState.idle,
        text: 'привет',
      ),
      injectReply: const DictaStatus(
        connected: true,
        state: DaemonState.idle,
        error: 'нет активного окна',
      ),
    );
    await tester.pumpWidget(buildFakeApp(fake, onCopy: (_) {}));
    await tick(tester);
    await captureTextViaStop(tester);
    await goToHistory(tester);

    await tester.tap(find.byKey(const Key('histInjectButton-1')));
    await tester.pump();
    await tester.pump();
    await tester.pump();

    // dashInjectFailed heading + verbatim daemon body (last_text_card canon).
    expect(find.text('ошибка вставки'), findsOneWidget);
    expect(find.text('нет активного окна'), findsOneWidget);
    expect(find.text('текст вставлен'), findsNothing);

    await tester.pumpWidget(const SizedBox.shrink());
  });

  testWidgets('row buttons express priority: inject tonal, copy elevated, delete outlined-error',
      (tester) async {
    final fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.recording),
      stopReply: const DictaStatus(
        connected: true,
        state: DaemonState.idle,
        text: 'привет',
      ),
    );
    await tester.pumpWidget(buildFakeApp(fake, onCopy: (_) {}));
    await tick(tester);
    await captureTextViaStop(tester);
    await goToHistory(tester);

    // The priority action is tonal, copy stays the plain elevated default,
    // delete is outlined with the error foreground.
    expect(
      tester.widget<FilledButton>(find.byKey(const Key('histInjectButton-1'))),
      isA<FilledButton>(),
    );
    expect(
      tester.widget<ElevatedButton>(find.byKey(const Key('histCopyButton-1'))),
      isA<ElevatedButton>(),
    );
    final delete = tester.widget<OutlinedButton>(
        find.byKey(const Key('histDeleteButton-1')));
    final error = Theme.of(tester.element(find.byType(HistoryScreen)))
        .colorScheme
        .error;
    final states = <WidgetState>{};
    expect(delete.style?.foregroundColor?.resolve(states), error);
    expect(delete.style?.side?.resolve(states)?.color, error);

    await tester.pumpWidget(const SizedBox.shrink());
  });

  testWidgets('search field exposes one merged semantics node with its label',
      (tester) async {
    final fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.idle),
    );
    await tester.pumpWidget(buildFakeApp(fake, onCopy: (_) {}));
    await tick(tester);
    await goToHistory(tester);

    final handle = tester.ensureSemantics();
    // WR-01: MergeSemantics folds the mandated label and the localized hint
    // into ONE editable node ("поиск по журналу\nпоиск по тексту"). Flutter
    // merges labels, so match by pattern as its finder docs recommend.
    expect(find.bySemanticsLabel(RegExp(r'^поиск по журналу')), findsOneWidget);
    final root = tester.binding.renderViews.first.owner!.semanticsOwner!
        .rootSemanticsNode!;
    final labelled = <SemanticsNode>[];
    void collect(SemanticsNode node) {
      if (node.label.startsWith('поиск по журналу')) labelled.add(node);
      node.visitChildren((child) {
        collect(child);
        return true;
      });
    }

    collect(root);
    expect(labelled, hasLength(1));
    // The node carrying the label is the editable field itself: one focus stop.
    expect(labelled.single.hasFlag(SemanticsFlag.isTextField), isTrue);

    handle.dispose();
    await tester.pumpWidget(const SizedBox.shrink());
  });

  testWidgets('unmount during in-flight inject leaves the dead context alone',
      (tester) async {
    final gate = Completer<void>();
    final fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.recording),
      stopReply: const DictaStatus(
        connected: true,
        state: DaemonState.idle,
        text: 'привет',
      ),
      injectGate: gate,
    );
    final showShell = ValueNotifier<bool>(true);
    addTearDown(showShell.dispose);
    Widget app() => buildFakeApp(
      fake,
      onCopy: (_) {},
      home: ValueListenableBuilder<bool>(
        valueListenable: showShell,
        builder: (_, visible, _) =>
            visible ? const ShellWidget() : const SizedBox.shrink(),
      ),
    );
    await tester.pumpWidget(app());
    await tick(tester);
    await captureTextViaStop(tester);
    await goToHistory(tester);

    await tester.tap(find.byKey(const Key('histInjectButton-1')));
    await tester.pump();
    expect(fake.injectTexts, <String>['привет']);

    // The shell uses IndexedStack, so switching tabs keeps HistoryScreen
    // mounted. Swap only the shell subtree (ProviderScope stays alive, so the
    // notifier can still settle) to deactivate this context and exercise
    // `if (!context.mounted) return;` after the inject await.
    showShell.value = false;
    await tester.pump();
    gate.complete();
    await tester.pump();
    await tester.pump();

    // The guard returns before showSnackBar on the unmounted context.
    expect(tester.takeException(), isNull);
    expect(find.text('текст вставлен'), findsNothing);
  });

  testWidgets('empty journal states the honest A4 limitation', (tester) async {
    final fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.idle),
    );
    await tester.pumpWidget(buildFakeApp(fake, onCopy: (_) {}));
    await tick(tester);
    await goToHistory(tester);

    expect(find.text('журнал пуст'), findsOneWidget);
    expect(
      find.text(
        'записи появляются после диктовок из приложения — глобальный хоткей демона приложению не виден',
      ),
      findsOneWidget,
    );

    await tester.pumpWidget(const SizedBox.shrink());
  });

  testWidgets('long entries collapse to 3 lines and expand on tap',
      (tester) async {
    final longText = 'длинная запись ' * 20; // > 160 characters
    final fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.recording),
      stopReply: DictaStatus(
        connected: true,
        state: DaemonState.idle,
        text: longText,
      ),
    );
    await tester.pumpWidget(buildFakeApp(fake, onCopy: (_) {}));
    await tick(tester);
    await captureTextViaStop(tester);
    await goToHistory(tester);

    final button = find.byKey(const Key('histExpandButton-1'));
    expect(button, findsOneWidget);
    expect(find.text('показать полностью'), findsOneWidget);
    SelectableText body() =>
        tester.widget<SelectableText>(find.byType(SelectableText).first);
    expect(body().maxLines, 3);

    await tester.tap(button);
    await tester.pump();
    expect(find.text('свернуть'), findsOneWidget);
    expect(body().maxLines, isNull);

    await tester.tap(button);
    await tester.pump();
    expect(find.text('показать полностью'), findsOneWidget);
    expect(body().maxLines, 3);

    await tester.pumpWidget(const SizedBox.shrink());
  });
}
