import 'package:feel/src/pages/home/src/feed.dart';
import 'package:feel/src/pages/home/src/push.dart';
import 'package:flutter/material.dart';

class HomePage extends StatefulWidget {
  const HomePage({
    super.key,
  });

  @override
  State<HomePage> createState() => _HomePageState();
}

class _HomePageState extends State<HomePage> {
  int currentIndex = 0;

  void onTabChanged(int value) {
    currentIndex = value;
    setState(() {});
  }

  static const List<Widget> _pages = <Widget>[
    PushPage(),
    PushPage(),
    FeedPage(),
    PushPage(),
  ];

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: IndexedStack(
        index: currentIndex,
        children: _pages,
      ),
      bottomNavigationBar: BottomNavigationBar(
        items: const [
          BottomNavigationBarItem(icon: Icon(Icons.home), label: "首页"),
          BottomNavigationBarItem(icon: Icon(Icons.home), label: "动态"),
          BottomNavigationBarItem(icon: Icon(Icons.chat), label: "聊天"),
          BottomNavigationBarItem(icon: Icon(Icons.home), label: "我的"),
        ],
        backgroundColor: Colors.black,
        // 选中颜色
        selectedItemColor: Colors.orange,
        // 固定颜色
        // fixedColor:Colors.orange,
        // 未选中颜色
        unselectedItemColor: Colors.lightGreen,
        // 选中图标主题
        selectedIconTheme: const IconThemeData(
          // 图标颜色
          color: Colors.red,
          // 图标大小
          size: 32,
          // 图标透明度
          opacity: 1.0,
        ),
        // 未选中图标主题
        unselectedIconTheme: const IconThemeData(
          color: Colors.blue,
          size: 24,
          opacity: 0.5,
        ),
        onTap: onTabChanged,
        currentIndex: currentIndex,
      ),
    );
  }
}
