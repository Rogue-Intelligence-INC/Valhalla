import 'package:flutter/material.dart';

import '../models/chat_message.dart';

class MessageBubble extends StatelessWidget {
  const MessageBubble({super.key, required this.message});

  final ChatMessage message;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final isUser = message.role == ChatRole.user;
    final isSystem = message.role == ChatRole.system;
    final isProactive = message.role == ChatRole.proactive || message.isProactive;

    Color bg;
    Color fg;
    Alignment align;
    IconData? badgeIcon;

    if (isSystem) {
      bg = theme.colorScheme.surfaceContainerHighest.withOpacity(0.6);
      fg = theme.colorScheme.onSurfaceVariant;
      align = Alignment.center;
    } else if (isProactive) {
      bg = const Color(0xFF1A2332);
      fg = const Color(0xFF7EE787);
      align = Alignment.centerLeft;
      badgeIcon = Icons.bolt;
    } else if (isUser) {
      bg = theme.colorScheme.primaryContainer;
      fg = theme.colorScheme.onPrimaryContainer;
      align = Alignment.centerRight;
    } else {
      bg = message.error
          ? theme.colorScheme.errorContainer
          : theme.colorScheme.secondaryContainer;
      fg = message.error
          ? theme.colorScheme.onErrorContainer
          : theme.colorScheme.onSecondaryContainer;
      align = Alignment.centerLeft;
    }

    return Align(
      alignment: align,
      child: ConstrainedBox(
        constraints: const BoxConstraints(maxWidth: 680),
        child: Card(
          color: bg,
          elevation: 0,
          margin: const EdgeInsets.symmetric(vertical: 4, horizontal: 12),
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(12),
            side: isProactive
                ? const BorderSide(color: Color(0xFF7EE787), width: 0.6)
                : BorderSide.none,
          ),
          child: Padding(
            padding: const EdgeInsets.all(12),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                if (badgeIcon != null || message.hasMultimodal)
                  Padding(
                    padding: const EdgeInsets.only(bottom: 6),
                    child: Wrap(
                      spacing: 6,
                      children: [
                        if (badgeIcon != null)
                          _Tag(
                            icon: badgeIcon,
                            label: 'proactive',
                            color: const Color(0xFF7EE787),
                          ),
                        if (message.hasMultimodal)
                          _Tag(
                            icon: Icons.layers,
                            label: message.modalities.join('+'),
                            color: const Color(0xFF06B6D4),
                          ),
                      ],
                    ),
                  ),
                SelectableText(
                  message.text,
                  style: theme.textTheme.bodyLarge?.copyWith(color: fg),
                ),
                if (message.decodePath != null || message.fogDecision != null) ...[
                  const SizedBox(height: 6),
                  Text(
                    [
                      if (message.decodePath != null) message.decodePath,
                      if (message.fogDecision != null) 'fog=${message.fogDecision}',
                    ].join(' · '),
                    style: theme.textTheme.labelSmall?.copyWith(
                      color: fg.withOpacity(0.7),
                      fontFamily: 'monospace',
                    ),
                  ),
                ],
              ],
            ),
          ),
        ),
      ),
    );
  }
}

class _Tag extends StatelessWidget {
  const _Tag({
    required this.icon,
    required this.label,
    required this.color,
  });

  final IconData icon;
  final String label;
  final Color color;

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
      decoration: BoxDecoration(
        color: color.withOpacity(0.15),
        borderRadius: BorderRadius.circular(999),
        border: Border.all(color: color.withOpacity(0.5)),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(icon, size: 12, color: color),
          const SizedBox(width: 4),
          Text(
            label,
            style: TextStyle(fontSize: 10, color: color, fontWeight: FontWeight.w600),
          ),
        ],
      ),
    );
  }
}
