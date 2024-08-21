/*
加载界面用于从其他系统同步数据
- 文件系统
- 网络
 */

import 'package:feel/src/store/index.dart';
import 'package:flutter/material.dart';

class SplashPage extends StatefulWidget {
  const SplashPage({
    super.key,
  });

  @override
  State<SplashPage> createState() => _SplashPageState();
}

class _SplashPageState extends State<SplashPage> {
  Future<void> bootstrap() async {
    ///全局实例注入
    ///从文件系统读取持久化数据
    await initStore();

    ///从网络同步数据
  }

  @override
  void initState() {
    super.initState();

    WidgetsBinding.instance.addPostFrameCallback((_) {
      bootstrap();
    });
  }

  @override
  Widget build(BuildContext context) {
    return Container(
      color: Colors.red,
    );
  }
}
