enum ChatRole { user, assistant, system, proactive }

class ChatMessage {
  const ChatMessage({
    required this.id,
    required this.role,
    required this.text,
    required this.timestamp,
    this.decodePath,
    this.error = false,
    this.isProactive = false,
    this.hasMultimodal = false,
    this.fogDecision,
    this.modalities = const [],
  });

  final String id;
  final ChatRole role;
  final String text;
  final DateTime timestamp;
  final String? decodePath;
  final bool error;
  final bool isProactive;
  final bool hasMultimodal;
  final String? fogDecision;
  final List<String> modalities;

  ChatMessage copyWith({
    String? text,
    String? decodePath,
    bool? error,
  }) {
    return ChatMessage(
      id: id,
      role: role,
      text: text ?? this.text,
      timestamp: timestamp,
      decodePath: decodePath ?? this.decodePath,
      error: error ?? this.error,
      isProactive: isProactive,
      hasMultimodal: hasMultimodal,
      fogDecision: fogDecision,
      modalities: modalities,
    );
  }
}
