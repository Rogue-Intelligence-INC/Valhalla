import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import 'screens/chat_screen.dart';
import 'state/chat_controller.dart';

void main() {
  const defaultUrl = String.fromEnvironment(
    'VALHALLA_BASE_URL',
    defaultValue: 'http://127.0.0.1:8787',
  );
  runApp(ValhallaChatApp(baseUrl: defaultUrl));
}

class ValhallaChatApp extends StatelessWidget {
  const ValhallaChatApp({super.key, required this.baseUrl});

  final String baseUrl;

  @override
  Widget build(BuildContext context) {
    return ChangeNotifierProvider(
      create: (_) => ChatController.withBaseUrl(baseUrl),
      child: MaterialApp(
        title: 'ValhallaBase Console',
        debugShowCheckedModeBanner: false,
        theme: ThemeData(
          colorScheme: ColorScheme.fromSeed(
            seedColor: const Color(0xFF3D9EFF),
            brightness: Brightness.dark,
            surface: const Color(0xFF121820),
          ),
          scaffoldBackgroundColor: const Color(0xFF0A0E14),
          cardTheme: CardThemeData(
            color: const Color(0xFF141820),
            elevation: 0,
            shape: RoundedRectangleBorder(
              borderRadius: BorderRadius.circular(10),
              side: BorderSide(
                color: Colors.white.withOpacity(0.08),
              ),
            ),
          ),
          useMaterial3: true,
          fontFamily: 'Segoe UI',
        ),
        home: const ChatScreen(),
      ),
    );
  }
}
