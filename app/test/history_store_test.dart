// In-memory history store contract (HIST-01/HIST-02).
// Newest-first ordering, stable ids, local removal, HH:mm formatter and the
// default empty search query.
import 'package:dicta_app/features/history/history.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:intl/date_symbol_data_local.dart';

void main() {
  setUpAll(() async {
    // Widget suites load ru date symbols via localizationsDelegates; a pure
    // unit test has to load them explicitly before DateFormat.Hm('ru').
    await initializeDateFormatting('ru');
  });

  test('add puts the newest entry on top with monotonic ids from 1', () {
    final container = ProviderContainer();
    addTearDown(container.dispose);
    final history = container.read(historyProvider.notifier);

    expect(container.read(historyProvider), isEmpty);

    history.add('первый');
    history.add('второй');

    final entries = container.read(historyProvider);
    expect(entries.length, 2);
    expect(entries[0].text, 'второй');
    expect(entries[1].text, 'первый');
    expect(entries[0].id, 2);
    expect(entries[1].id, 1);
    expect(entries[0].at, isA<DateTime>());
  });

  test('remove deletes only the target id; unknown ids are a no-op', () {
    final container = ProviderContainer();
    addTearDown(container.dispose);
    final history = container.read(historyProvider.notifier);

    history.add('a');
    history.add('b');
    history.add('c');
    expect(
      container.read(historyProvider).map((e) => e.id).toList(),
      <int>[3, 2, 1],
    );

    history.remove(2);
    expect(
      container.read(historyProvider).map((e) => e.id).toList(),
      <int>[3, 1],
    );

    history.remove(999);
    expect(
      container.read(historyProvider).map((e) => e.id).toList(),
      <int>[3, 1],
    );
  });

  test('head-derived ids are reused after removal but unique among live entries',
      () {
    final container = ProviderContainer();
    addTearDown(container.dispose);
    final history = container.read(historyProvider.notifier);

    history.add('a'); // id 1
    history.add('b'); // id 2
    history.remove(2); // live ids: [1]
    history.add('c'); // head 1 -> id 2 reused, no live clash

    final entries = container.read(historyProvider);
    expect(entries.map((e) => e.id).toList(), <int>[2, 1]);
    expect(entries.map((e) => e.text).toList(), <String>['c', 'a']);
    expect(entries.length, entries.map((e) => e.id).toSet().length);
  });

  test('formatEntryTime renders HH:mm for the ru locale', () {
    expect(formatEntryTime(DateTime(2026, 1, 1, 9, 5), 'ru'), '09:05');
    expect(formatEntryTime(DateTime(2026, 11, 22, 23, 59), 'ru'), '23:59');
  });

  test('historyQueryProvider starts with an empty query', () {
    final container = ProviderContainer();
    addTearDown(container.dispose);

    expect(container.read(historyQueryProvider), '');

    container.read(historyQueryProvider.notifier).set('мир');
    expect(container.read(historyQueryProvider), 'мир');
  });
}
