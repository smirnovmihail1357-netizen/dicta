/// Hugging Face browser: verified catalog, explicit search/tree fetches,
/// UI-only VRAM limit, validated direct link and the shared download panel.
/// All transport goes through hfClientProvider / downloadManagerProvider;
/// there is no direct network transport and no automatic fetch under features/.
library;

import 'package:dicta_app/core/downloads.dart';
import 'package:dicta_app/core/hf/hf_catalog.dart';
import 'package:dicta_app/core/hf/hf_client.dart';
import 'package:dicta_app/core/hf/hf_parse.dart';
import 'package:dicta_app/core/hf/hf_urls.dart';
import 'package:dicta_app/core/providers.dart';
import 'package:dicta_app/core/theme_ext.dart';
import 'package:dicta_app/l10n/generated/app_localizations.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

class HfBrowser extends ConsumerStatefulWidget {
  const HfBrowser({super.key});

  @override
  ConsumerState<HfBrowser> createState() => _HfBrowserState();
}

class _HfBrowserState extends ConsumerState<HfBrowser> {
  String _query = '';
  String _sort = 'downloads';
  bool _asrOnly = false;
  int _offset = 0;
  List<HfRepo> _repos = const <HfRepo>[];
  String? _repo;
  String _subpath = '';
  List<HfFile> _files = const <HfFile>[];
  List<String> _dirs = const <String>[];
  String _directUrl = '';
  String? _fetchError;
  bool _searched = false;

  void _snack(String message) {
    if (!mounted) return;
    ScaffoldMessenger.of(context).showSnackBar(SnackBar(content: Text(message)));
  }

  Future<void> _startDownload({
    required String url,
    required String destName,
    int? expectedBytes,
  }) async {
    await ref.read(downloadManagerProvider.notifier).start(
          url: url,
          destName: destName,
          expectedBytes: expectedBytes,
        );
  }

  Future<void> _search() async {
    final url = browseUrl(_query, _sort, _asrOnly, _offset);
    try {
      final body = await ref.read(hfClientProvider).fetchText(url);
      if (!mounted) return;
      setState(() {
        _repos = parseSearch(body);
        _fetchError = null;
        _searched = true;
      });
    } on HfHttpException catch (error) {
      if (!mounted) return;
      setState(() {
        _fetchError = error.statusCode == 401
            ? AppLocalizations.of(context)!.modelRepoClosed
            : 'HTTP ${error.statusCode}';
      });
    }
  }

  Future<void> _openTree(String repo, String subpath) async {
    final url = treeUrlAt(repo, subpath);
    try {
      final body = await ref.read(hfClientProvider).fetchText(url);
      if (!mounted) return;
      setState(() {
        _repo = repo;
        _subpath = subpath;
        _files = parseTree(body);
        _dirs = parseTreeDirs(body);
        _fetchError = null;
      });
    } on HfHttpException catch (error) {
      if (!mounted) return;
      setState(() {
        _fetchError = error.statusCode == 401
            ? AppLocalizations.of(context)!.modelRepoClosed
            : 'HTTP ${error.statusCode}';
      });
    }
  }

  void _goBack() {
    if (_subpath.isEmpty) return;
    final segments = _subpath.split('/');
    segments.removeLast();
    _openTree(_repo!, segments.join('/'));
  }

  String _sizeLabel(int bytes, AppLocalizations l10n) {
    if (bytes >= 1000000000) {
      return '${(bytes / 1000000000).toStringAsFixed(1)} GB';
    }
    return '${(bytes / 1000000).toStringAsFixed(0)} MB';
  }

  String _noteFor(int index, AppLocalizations l10n) => switch (index) {
        0 => l10n.modelNoteTiny,
        1 => l10n.modelNoteBase,
        2 => l10n.modelNoteSmall,
        3 => l10n.modelNoteMedium,
        4 => l10n.modelNoteLargeTurbo,
        _ => l10n.modelNoteLarge,
      };

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    final colors = Theme.of(context).colorScheme;
    final settings = ref.watch(settingsSnapshotProvider).value;
    final installed = settings?.models ?? const <String>[];
    final vram = ref.watch(vramLimitGbProvider);
    final download = ref.watch(downloadManagerProvider);
    final busy = download is DownloadActive;

    ref.listen(downloadManagerProvider, (previous, next) {
      if (previous == next) return;
      if (next is DownloadDone && !next.alreadyExisted) {
        _snack(l10n.modelDownloadDone);
      } else if (next is DownloadCancelled) {
        _snack(l10n.modelDownloadCancelled);
      } else if (next is DownloadFailed) {
        _snack('${l10n.modelDownloadFailed}: ${next.message}');
      }
    });

    final directUri = Uri.tryParse(_directUrl.trim());
    final directValid = directUri != null &&
        (directUri.scheme == 'http' || directUri.scheme == 'https') &&
        filenameOf(_directUrl.trim()) != null;
    final directName = directValid ? filenameOf(_directUrl.trim()) : null;

    Widget downloadButton({
      required String url,
      required String destName,
      int? expectedBytes,
      bool blocked = false,
    }) =>
        FilledButton(
          key: Key('modelDownloadButton-$destName'),
          onPressed: busy || blocked
              ? null
              : () => _startDownload(
                    url: url,
                    destName: destName,
                    expectedBytes: expectedBytes,
                  ),
          child: Text(l10n.modelDownloadButton),
        );

    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        const SizedBox(height: 20),
        Text(
          l10n.modelCatalogTitle,
          style: Theme.of(context).textTheme.titleMedium,
        ),
        const SizedBox(height: 8),
        Row(
          children: [
            Expanded(child: Text(l10n.modelVramLabel)),
            SizedBox(
              width: 180,
              child: Slider(
                key: const Key('modelVramSlider'),
                min: 2,
                max: 32,
                divisions: 30,
                value: vram,
                label: vram.toInt().toString(),
                // UI-only: never an engine.set (there is no daemon key).
                onChanged: (value) =>
                    ref.read(vramLimitGbProvider.notifier).set(value),
              ),
            ),
            Text('${vram.toInt()}', style: AppTextStyles.mutedOf(context)),
          ],
        ),
        ...kCatalog.asMap().entries.map((entry) {
          final item = entry.value;
          final exists = installed.any((path) => destName(path) == item.file);
          final over = vramBlocked(item.approxBytes, vram);
          return Card(
            child: Padding(
              padding: const EdgeInsets.all(12),
              child: Row(
                children: [
                  Expanded(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text(item.file),
                        Text(
                          '${_sizeLabel(item.approxBytes, l10n)} · ${_noteFor(entry.key, l10n)}',
                          style: AppTextStyles.mutedOf(context).copyWith(
                            fontFeatures: const [FontFeature.tabularFigures()],
                          ),
                        ),
                        if (exists)
                          Text(
                            l10n.modelAlreadyInstalled,
                            style: AppTextStyles.mutedOf(context),
                          ),
                        if (over)
                          Text(
                            l10n.modelVramOver,
                            style: TextStyle(color: colors.error),
                          ),
                      ],
                    ),
                  ),
                  downloadButton(
                    url: urlFor(item.file),
                    destName: item.file,
                    expectedBytes: item.approxBytes,
                    blocked: exists || over,
                  ),
                ],
              ),
            ),
          );
        }),
        const SizedBox(height: 20),
        Text(
          l10n.modelSearchHint,
          style: Theme.of(context).textTheme.titleMedium,
        ),
        Padding(
          padding: const EdgeInsets.symmetric(vertical: 8),
          child: MergeSemantics(
            child: Semantics(
              label: l10n.modelSearchSemantics,
              child: TextField(
                key: const Key('modelSearchField'),
                decoration: InputDecoration(
                  hintText: l10n.modelSearchHint,
                  border: const OutlineInputBorder(),
                ),
                onChanged: (value) => _query = value,
              ),
            ),
          ),
        ),
        SegmentedButton<String>(
          key: const Key('modelSortSegments'),
          segments: [
            ButtonSegment(value: 'downloads', label: Text(l10n.modelSortDownloads)),
            ButtonSegment(value: 'likes', label: Text(l10n.modelSortLikes)),
            ButtonSegment(
              value: 'lastModified',
              label: Text(l10n.modelSortModified),
            ),
          ],
          selected: {_sort},
          onSelectionChanged: (next) => setState(() => _sort = next.first),
        ),
        SwitchListTile(
          key: const Key('modelAsrSwitch'),
          title: Text(l10n.modelAsrFilter),
          value: _asrOnly,
          onChanged: (value) => setState(() => _asrOnly = value),
        ),
        Align(
          alignment: Alignment.centerLeft,
          child: FilledButton(
            key: const Key('modelSearchButton'),
            onPressed: _search,
            child: Text(l10n.modelSearchButton),
          ),
        ),
        if (_fetchError != null) ...[
          const SizedBox(height: 8),
          Text(
            _fetchError!,
            key: const Key('modelFetchError'),
            style: TextStyle(color: colors.error),
          ),
        ],
        if (_repos.isNotEmpty) ...[
          ..._repos.map(
            (repo) => ListTile(
              key: Key('modelRepo-${repo.id}'),
              title: Text(repo.id),
              subtitle: Text(
                '${repo.downloads} · ${repo.likes}',
                style: AppTextStyles.mutedOf(context).copyWith(
                  fontFeatures: const [FontFeature.tabularFigures()],
                ),
              ),
              onTap: () => _openTree(repo.id, ''),
            ),
          ),
          Align(
            alignment: Alignment.centerLeft,
            child: TextButton(
              key: const Key('modelShowMoreButton'),
              onPressed: () {
                _offset += 20;
                _search();
              },
              child: Text(l10n.modelShowMore),
            ),
          ),
        ] else if (_searched && _fetchError == null) ...[
          const SizedBox(height: 8),
          Text(l10n.modelResultsEmpty, style: AppTextStyles.mutedOf(context)),
        ],
        if (_repo != null) ...[
          const SizedBox(height: 12),
          Row(
            children: [
              Text(_repo!),
              const Spacer(),
              TextButton(
                key: const Key('modelTreeBackButton'),
                onPressed: _subpath.isEmpty ? null : _goBack,
                child: Text(l10n.modelSubdirBack),
              ),
            ],
          ),
          if (_dirs.isNotEmpty)
            Wrap(
              children: _dirs
                  .map(
                    (dir) => Padding(
                      padding: const EdgeInsets.only(right: 8),
                      child: OutlinedButton(
                        key: Key('modelSubdir-$dir'),
                        onPressed: () => _openTree(_repo!, dir),
                        child: Text(destName(dir)),
                      ),
                    ),
                  )
                  .toList(),
            ),
          if (_files.isEmpty)
            Text(l10n.modelTreeEmpty, style: AppTextStyles.mutedOf(context))
          else
            ..._files.map(
              (file) => ListTile(
                key: Key('modelTreeFile-${file.path}'),
                title: Text(destName(file.path)),
                subtitle: Text(
                  vramBlocked(file.size, vram)
                      ? '${_sizeLabel(file.size, l10n)} · ${l10n.modelVramOver}'
                      : _sizeLabel(file.size, l10n),
                  style: AppTextStyles.mutedOf(context).copyWith(
                    fontFeatures: const [FontFeature.tabularFigures()],
                  ),
                ),
                trailing: downloadButton(
                  url: resolveUrl(_repo!, file.path),
                  destName: destName(file.path),
                  expectedBytes: file.size > 0 ? file.size : null,
                  blocked: vramBlocked(file.size, vram),
                ),
              ),
            ),
        ],
        const SizedBox(height: 20),
        Text(
          l10n.modelDirectLinkLabel,
          style: Theme.of(context).textTheme.titleMedium,
        ),
        Padding(
          padding: const EdgeInsets.symmetric(vertical: 8),
          child: MergeSemantics(
            child: Semantics(
              label: l10n.modelDirectLinkLabel,
              child: TextField(
                key: const Key('modelDirectLinkField'),
                decoration: InputDecoration(
                  hintText: l10n.modelDirectLinkHint,
                  border: const OutlineInputBorder(),
                ),
                onChanged: (value) => setState(() => _directUrl = value),
              ),
            ),
          ),
        ),
        if (directName != null)
          Text(
            directName,
            key: const Key('modelDirectName'),
            style: AppTextStyles.mutedOf(context),
          ),
        Align(
          alignment: Alignment.centerLeft,
          child: FilledButton(
            key: const Key('modelDirectDownloadButton'),
            onPressed: busy || directName == null
                ? null
                : () => _startDownload(
                      url: _directUrl.trim(),
                      destName: directName,
                    ),
            child: Text(l10n.modelDownloadButton),
          ),
        ),
        const SizedBox(height: 12),
        if (download is DownloadActive) ...[
          Semantics(
            liveRegion: true,
            child: Column(
              key: const Key('modelDownloadPanel'),
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                if (download.expectedBytes != null &&
                    download.expectedBytes! > 0)
                  LinearProgressIndicator(
                    value: (download.received / download.expectedBytes!)
                        .clamp(0.0, 1.0)
                        .toDouble(),
                    minHeight: 8,
                  )
                else
                  const LinearProgressIndicator(minHeight: 8),
                const SizedBox(height: 4),
                Text(
                  l10n.modelDownloadingMb(
                    (download.received / 1000000).toStringAsFixed(0),
                  ),
                  style: AppTextStyles.mutedOf(context).copyWith(
                    fontFeatures: const [FontFeature.tabularFigures()],
                  ),
                ),
                Align(
                  alignment: Alignment.centerLeft,
                  child: OutlinedButton(
                    key: const Key('modelCancelDownloadButton'),
                    style: OutlinedButton.styleFrom(
                      foregroundColor: colors.error,
                      side: BorderSide(color: colors.error),
                    ),
                    onPressed: () =>
                        ref.read(downloadManagerProvider.notifier).cancel(),
                    child: Text(l10n.modelCancelDownload),
                  ),
                ),
              ],
            ),
          ),
        ],
        const SizedBox(height: 20),
      ],
    );
  }
}
