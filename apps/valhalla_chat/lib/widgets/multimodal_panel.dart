import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:provider/provider.dart';

import '../state/chat_controller.dart';
import 'triad_panels.dart';

class MultimodalInputPanel extends StatefulWidget {
  const MultimodalInputPanel({super.key});

  @override
  State<MultimodalInputPanel> createState() => _MultimodalInputPanelState();
}

class _MultimodalInputPanelState extends State<MultimodalInputPanel> {
  late final _textCtrl = TextEditingController();
  late final _audioCtrl = TextEditingController();
  late final _imageCtrl = TextEditingController();

  @override
  void dispose() {
    _textCtrl.dispose();
    _audioCtrl.dispose();
    _imageCtrl.dispose();
    super.dispose();
  }

  void _syncFromController(ChatController ctrl) {
    _textCtrl.text = ctrl.multimodal.textNote;
    _audioCtrl.text = ctrl.multimodal.audioFeatures;
    _imageCtrl.text = ctrl.multimodal.imageFeatures;
  }

  @override
  Widget build(BuildContext context) {
    final ctrl = context.watch<ChatController>();
    final mm = ctrl.multimodal;

    return AnnotatedSection(
      title: 'MULTIMODAL INGRESS',
      annotationKey: 'multimodal',
      trailing: Switch(
        value: mm.enabled,
        onChanged: ctrl.loading ? null : ctrl.setMultimodalEnabled,
      ),
      child: Column(
        children: [
          TextField(
            enabled: mm.enabled && !ctrl.loading,
            controller: _textCtrl,
            decoration: const InputDecoration(
              labelText: 'Text note (context_items)',
              border: OutlineInputBorder(),
              isDense: true,
            ),
            onChanged: (v) {
              mm.textNote = v;
            },
          ),
          const SizedBox(height: 8),
          TextField(
            enabled: mm.enabled && !ctrl.loading,
            controller: _audioCtrl,
            decoration: const InputDecoration(
              labelText: 'audio_features (comma floats)',
              hintText: '0.12, 0.34, 0.56, 0.78',
              border: OutlineInputBorder(),
              isDense: true,
            ),
            onChanged: (v) => mm.audioFeatures = v,
          ),
          const SizedBox(height: 8),
          TextField(
            enabled: mm.enabled && !ctrl.loading,
            controller: _imageCtrl,
            decoration: const InputDecoration(
              labelText: 'image_features (comma floats)',
              hintText: '0.01, 0.02, 0.03, 0.04',
              border: OutlineInputBorder(),
              isDense: true,
            ),
            onChanged: (v) => mm.imageFeatures = v,
          ),
          const SizedBox(height: 8),
          OutlinedButton(
            onPressed: mm.enabled && !ctrl.loading
                ? () {
                    mm.audioFeatures = '0.12, 0.34, 0.56, 0.78';
                    mm.imageFeatures = '0.01, 0.02, 0.03, 0.04, 0.05, 0.06';
                    mm.textNote =
                        'Multimodal ingress: text + audio + image features.';
                    _syncFromController(ctrl);
                    ctrl.updateMultimodal(mm);
                  }
                : null,
            child: const Text('Load demo vectors'),
          ),
        ],
      ),
    );
  }
}
