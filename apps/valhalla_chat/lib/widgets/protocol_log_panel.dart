import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:provider/provider.dart';

import '../models/app_models.dart';
import '../state/chat_controller.dart';
import '../theme/valhalla_tokens.dart';

class ProtocolLogPanel extends StatelessWidget {
  const ProtocolLogPanel({super.key});

  @override
  Widget build(BuildContext context) {
    final ctrl = context.watch<ChatController>();
    final logs = ctrl.protocolLog;

    return Material(
      elevation: 4,
      color: const Color(0xFF0C0E12),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
            decoration: BoxDecoration(
              border: Border(
                bottom: BorderSide(
                  color: Theme.of(context).colorScheme.outline.withOpacity(0.35),
                ),
              ),
            ),
            child: Row(
              children: [
                const Icon(Icons.terminal, size: 18, color: Color(0xFF7EE787)),
                const SizedBox(width: 8),
                Text(
                  'PROTOCOL LOG · CLI mirror',
                  style: Theme.of(context).textTheme.labelMedium?.copyWith(
                        letterSpacing: 1,
                        fontWeight: FontWeight.w600,
                      ),
                ),
                const Spacer(),
                Tooltip(
                  message: kTriadAnnotations['protocol'] ?? '',
                  child: Icon(
                    Icons.info_outline,
                    size: 16,
                    color: Theme.of(context).colorScheme.outline,
                  ),
                ),
                IconButton(
                  icon: const Icon(Icons.delete_sweep, size: 18),
                  tooltip: 'Clear log',
                  onPressed: logs.isEmpty ? null : ctrl.clearProtocolLog,
                ),
              ],
            ),
          ),
          Expanded(
            child: logs.isEmpty
                ? Center(
                    child: Text(
                      'HTTP requests/responses appear here\n'
                      'with curl-style CLI mirror',
                      textAlign: TextAlign.center,
                      style: Theme.of(context).textTheme.bodySmall?.copyWith(
                            color: Theme.of(context).colorScheme.onSurfaceVariant,
                          ),
                    ),
                  )
                : ListView.builder(
                    padding: const EdgeInsets.all(8),
                    itemCount: logs.length,
                    itemBuilder: (_, i) => _LogTile(entry: logs[i]),
                  ),
          ),
        ],
      ),
    );
  }
}

class _LogTile extends StatefulWidget {
  const _LogTile({required this.entry});

  final ProtocolLogEntry entry;

  @override
  State<_LogTile> createState() => _LogTileState();
}

class _LogTileState extends State<_LogTile> {
  bool expanded = false;

  @override
  Widget build(BuildContext context) {
    final e = widget.entry;
    final isOut = e.direction == 'out';
    final color = e.error
        ? const Color(0xFFEF4444)
        : isOut
            ? const Color(0xFF3D9EFF)
            : const Color(0xFF7EE787);

    return Padding(
      padding: const EdgeInsets.only(bottom: 6),
      child: InkWell(
        onTap: () => setState(() => expanded = !expanded),
        borderRadius: BorderRadius.circular(6),
        child: Container(
          padding: const EdgeInsets.all(8),
          decoration: BoxDecoration(
            color: const Color(0xFF141820),
            borderRadius: BorderRadius.circular(6),
            border: Border.all(color: color.withOpacity(0.35)),
          ),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Row(
                children: [
                  Text(
                    isOut ? '→' : '←',
                    style: TextStyle(color: color, fontWeight: FontWeight.bold),
                  ),
                  const SizedBox(width: 6),
                  Expanded(
                    child: Text(
                      '${e.method} ${e.path} · ${e.summary}',
                      style: const TextStyle(
                        fontFamily: 'monospace',
                        fontSize: 11,
                      ),
                    ),
                  ),
                  if (e.durationMs != null)
                    Text(
                      '${e.durationMs}ms',
                      style: Theme.of(context).textTheme.labelSmall,
                    ),
                  IconButton(
                    icon: const Icon(Icons.copy, size: 14),
                    padding: EdgeInsets.zero,
                    constraints: const BoxConstraints(minWidth: 28, minHeight: 28),
                    onPressed: () {
                      Clipboard.setData(
                        ClipboardData(text: expanded ? e.prettyBody : e.cliLine),
                      );
                    },
                  ),
                ],
              ),
              if (expanded) ...[
                const SizedBox(height: 6),
                Text(
                  '# CLI',
                  style: Theme.of(context).textTheme.labelSmall?.copyWith(
                        color: const Color(0xFF8B93A7),
                      ),
                ),
                SelectableText(
                  e.cliLine.replaceAll(r'$BASE', 'http://127.0.0.1:8787'),
                  style: const TextStyle(
                    fontFamily: 'monospace',
                    fontSize: 10,
                    color: Color(0xFF8B93A7),
                  ),
                ),
                if (e.prettyBody.isNotEmpty) ...[
                  const SizedBox(height: 6),
                  Text(
                    '# JSON',
                    style: Theme.of(context).textTheme.labelSmall?.copyWith(
                          color: const Color(0xFF8B93A7),
                        ),
                  ),
                  SelectableText(
                    e.prettyBody.length > 4000
                        ? '${e.prettyBody.substring(0, 4000)}\n… (truncated)'
                        : e.prettyBody,
                    style: const TextStyle(
                      fontFamily: 'monospace',
                      fontSize: 10,
                      color: Color(0xFFE8EAED),
                    ),
                  ),
                ],
              ],
            ],
          ),
        ),
      ),
    );
  }
}
