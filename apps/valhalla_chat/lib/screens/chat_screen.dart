import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../state/chat_controller.dart';
import '../widgets/message_bubble.dart';
import '../widgets/multimodal_panel.dart';
import '../widgets/protocol_log_panel.dart';
import '../widgets/triad_panels.dart';

class ChatScreen extends StatefulWidget {
  const ChatScreen({super.key});

  @override
  State<ChatScreen> createState() => _ChatScreenState();
}

class _ChatScreenState extends State<ChatScreen> {
  final _input = TextEditingController();
  final _contextInput = TextEditingController(
    text: 'ValhallaBase primary protocol is valhalla-base-v1.\n'
        'Production decode mode is hybrid with native retrieval for open QA.\n'
        'Tier A RAGTruth score is 100 out of 100 grounded answers.',
  );
  final _scroll = ScrollController();
  final _urlInput = TextEditingController(text: 'http://127.0.0.1:8787');
  double _logHeight = 220;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      final ctrl = context.read<ChatController>();
      ctrl.checkConnection();
      ctrl.setSystemContext(_contextInput.text);
    });
  }

  @override
  void dispose() {
    _input.dispose();
    _contextInput.dispose();
    _urlInput.dispose();
    _scroll.dispose();
    super.dispose();
  }

  void _scrollToBottom() {
    if (!_scroll.hasClients) return;
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (_scroll.hasClients) {
        _scroll.jumpTo(_scroll.position.maxScrollExtent);
      }
    });
  }

  Future<void> _send() async {
    final ctrl = context.read<ChatController>();
    final text = _input.text;
    _input.clear();
    await ctrl.sendMessage(text);
    _scrollToBottom();
  }

  @override
  Widget build(BuildContext context) {
    final ctrl = context.watch<ChatController>();
    _scrollToBottom();

    return Scaffold(
      backgroundColor: const Color(0xFF0A0E14),
      appBar: AppBar(
        title: const Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text('ValhallaBase Console', style: TextStyle(fontSize: 17)),
            Text(
              'Triad · Fate · Hybrid decode',
              style: TextStyle(fontSize: 11, fontWeight: FontWeight.normal),
            ),
          ],
        ),
        actions: [
          if (ctrl.modelState.gitSha.isNotEmpty)
            Padding(
              padding: const EdgeInsets.only(right: 8),
              child: Center(
                child: Text(
                  ctrl.modelState.gitSha,
                  style: const TextStyle(fontFamily: 'monospace', fontSize: 11),
                ),
              ),
            ),
          IconButton(
            tooltip: 'Toggle protocol log',
            onPressed: () => ctrl.setShowProtocolLog(!ctrl.showProtocolLog),
            icon: Icon(
              ctrl.showProtocolLog ? Icons.terminal : Icons.terminal_outlined,
            ),
          ),
          IconButton(
            tooltip: 'Connection',
            onPressed: ctrl.loading ? null : () => ctrl.checkConnection(),
            icon: Icon(
              ctrl.connected ? Icons.cloud_done : Icons.cloud_off,
              color: ctrl.connected ? const Color(0xFF7EE787) : null,
            ),
          ),
          IconButton(
            tooltip: 'Clear chat',
            onPressed: ctrl.loading ? null : () => ctrl.clearChat(),
            icon: const Icon(Icons.delete_outline),
          ),
        ],
      ),
      drawer: _SettingsDrawer(
        urlInput: _urlInput,
        contextInput: _contextInput,
      ),
      body: Column(
        children: [
          if (!ctrl.connected)
            MaterialBanner(
              backgroundColor: const Color(0xFF1A2332),
              content: Text(
                ctrl.connectionError ??
                    'Start: valhalla_base serve --with-dashboard --port 8787',
              ),
              leading: const Icon(Icons.warning_amber),
              actions: [
                TextButton(onPressed: ctrl.checkConnection, child: const Text('Retry')),
              ],
            ),
          Expanded(
            child: LayoutBuilder(
              builder: (context, constraints) {
                final wide = constraints.maxWidth >= 960;
                if (wide) {
                  return Row(
                    crossAxisAlignment: CrossAxisAlignment.stretch,
                    children: [
                      Expanded(flex: 5, child: _ChatColumn(
                        scroll: _scroll,
                        input: _input,
                        onSend: _send,
                      )),
                      VerticalDivider(
                        width: 1,
                        color: Theme.of(context).colorScheme.outline.withOpacity(0.25),
                      ),
                      SizedBox(
                        width: 380,
                        child: _TriadSidebar(),
                      ),
                    ],
                  );
                }
                return DefaultTabController(
                  length: 2,
                  child: Column(
                    children: [
                      const TabBar(
                        tabs: [
                          Tab(text: 'Chat'),
                          Tab(text: 'Triad / State'),
                        ],
                      ),
                      Expanded(
                        child: TabBarView(
                          children: [
                            _ChatColumn(
                              scroll: _scroll,
                              input: _input,
                              onSend: _send,
                            ),
                            SingleChildScrollView(
                              padding: const EdgeInsets.all(12),
                              child: _TriadSidebar(),
                            ),
                          ],
                        ),
                      ),
                    ],
                  ),
                );
              },
            ),
          ),
          if (ctrl.loading) const LinearProgressIndicator(minHeight: 2),
          if (ctrl.showProtocolLog)
            SizedBox(
              height: _logHeight,
              child: Stack(
                children: [
                  const ProtocolLogPanel(),
                  Positioned(
                    top: 0,
                    left: 0,
                    right: 0,
                    child: MouseRegion(
                      cursor: SystemMouseCursors.resizeRow,
                      child: GestureDetector(
                        onVerticalDragUpdate: (d) {
                          setState(() {
                            _logHeight = (_logHeight - d.delta.dy).clamp(120.0, 480.0);
                          });
                        },
                        child: Container(
                          height: 6,
                          color: Colors.transparent,
                        ),
                      ),
                    ),
                  ),
                ],
              ),
            ),
        ],
      ),
    );
  }
}

class _ChatColumn extends StatelessWidget {
  const _ChatColumn({
    required this.scroll,
    required this.input,
    required this.onSend,
  });

  final ScrollController scroll;
  final TextEditingController input;
  final VoidCallback onSend;

  @override
  Widget build(BuildContext context) {
    final ctrl = context.watch<ChatController>();

    return Column(
      children: [
        Expanded(
          child: ctrl.messages.isEmpty
              ? Center(
                  child: Padding(
                    padding: const EdgeInsets.all(24),
                    child: Text(
                      'ValhallaBase interactive console\n'
                      'Chat · Proactive · Multimodal · Triad viz',
                      textAlign: TextAlign.center,
                      style: Theme.of(context).textTheme.bodyLarge?.copyWith(
                            color: Theme.of(context).colorScheme.onSurfaceVariant,
                          ),
                    ),
                  ),
                )
              : ListView.builder(
                  controller: scroll,
                  padding: const EdgeInsets.symmetric(vertical: 12),
                  itemCount: ctrl.messages.length,
                  itemBuilder: (_, i) => MessageBubble(message: ctrl.messages[i]),
                ),
        ),
        SafeArea(
          child: Padding(
            padding: const EdgeInsets.fromLTRB(12, 8, 12, 12),
            child: Row(
              children: [
                Expanded(
                  child: TextField(
                    controller: input,
                    minLines: 1,
                    maxLines: 4,
                    textInputAction: TextInputAction.send,
                    onSubmitted: (_) => onSend(),
                    decoration: InputDecoration(
                      hintText: 'Message ValhallaBase…',
                      filled: true,
                      fillColor: const Color(0xFF141820),
                      border: OutlineInputBorder(
                        borderRadius: BorderRadius.circular(24),
                        borderSide: BorderSide.none,
                      ),
                      contentPadding: const EdgeInsets.symmetric(
                        horizontal: 16,
                        vertical: 12,
                      ),
                    ),
                  ),
                ),
                const SizedBox(width: 8),
                IconButton.filled(
                  onPressed: ctrl.loading ? null : onSend,
                  icon: const Icon(Icons.send),
                ),
              ],
            ),
          ),
        ),
      ],
    );
  }
}

class _TriadSidebar extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    final ctrl = context.watch<ChatController>();
    final ms = ctrl.modelState;

    return SingleChildScrollView(
      padding: const EdgeInsets.all(12),
      child: Column(
        children: [
          ModelStatusPanel(state: ms),
          const SizedBox(height: 10),
          HubQuadPanel(prefs: ms.hubPrefs, routed: ms.hubRouted),
          const SizedBox(height: 10),
          TileStemPanel(modelState: ms, dashboard: ctrl.dashboardViz),
          const SizedBox(height: 10),
          ProactivePanel(
            state: ctrl.proactiveState,
            modelState: ms,
            autoAfterTurn: ctrl.autoProactiveAfterTurn,
            loading: ctrl.loading,
            onToggle: ctrl.setProactiveEnabled,
            onAutoToggle: ctrl.setAutoProactiveAfterTurn,
            onTrigger: () => ctrl.triggerProactive(),
          ),
          const SizedBox(height: 10),
          const MultimodalInputPanel(),
          if (ctrl.dashboardViz.available) ...[
            const SizedBox(height: 10),
            AnnotatedSection(
              title: 'DASHBOARD SMOKE',
              annotationKey: 'hub',
              child: Text(
                'Tile neurons: ${ctrl.dashboardViz.tileNeurons.length} · '
                'Stem organs: ${ctrl.dashboardViz.stemOrgans.length} · '
                'pass=${ctrl.dashboardViz.overallPass}',
                style: Theme.of(context).textTheme.bodySmall,
              ),
            ),
          ],
        ],
      ),
    );
  }
}

class _SettingsDrawer extends StatelessWidget {
  const _SettingsDrawer({
    required this.urlInput,
    required this.contextInput,
  });

  final TextEditingController urlInput;
  final TextEditingController contextInput;

  @override
  Widget build(BuildContext context) {
    final ctrl = context.watch<ChatController>();

    return Drawer(
      child: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          Text('Settings', style: Theme.of(context).textTheme.titleLarge),
          const SizedBox(height: 12),
          TextField(
            controller: urlInput,
            decoration: const InputDecoration(
              labelText: 'API base URL',
              border: OutlineInputBorder(),
            ),
            onChanged: ctrl.setBaseUrl,
          ),
          const SizedBox(height: 8),
          SwitchListTile(
            title: const Text('Core fusion'),
            subtitle: const Text('config.core_fusion — structure→LM fuse'),
            value: ctrl.coreFusion,
            onChanged: ctrl.loading ? null : ctrl.setCoreFusion,
          ),
          const Divider(height: 24),
          Text('System context', style: Theme.of(context).textTheme.titleMedium),
          const SizedBox(height: 8),
          TextField(
            controller: contextInput,
            maxLines: 8,
            decoration: const InputDecoration(
              border: OutlineInputBorder(),
              hintText: 'One fact per line — first turn context',
            ),
            onChanged: ctrl.setSystemContext,
          ),
          const SizedBox(height: 12),
          FilledButton.tonalIcon(
            onPressed: ctrl.loading ? null : () => ctrl.warmup(),
            icon: const Icon(Icons.auto_fix_high),
            label: const Text('Fate warmup'),
          ),
        ],
      ),
    );
  }
}
