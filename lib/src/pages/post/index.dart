import 'package:feel/src/apis/post/index.dart';
import 'package:feel/src/apis/post/model.dart';
import 'package:feel/src/components/loading/page_loading.dart';
import 'package:feel/src/components/user_nav.dart';
import 'package:flutter/material.dart';
import 'package:get/get.dart';

class PostPage extends StatefulWidget {
  const PostPage({
    super.key,
  });

  @override
  State<PostPage> createState() => _PostPageState();
}

class _PostPageState extends State<PostPage> {
  Post? post;
  bool isInit = false;

  Future<void> init() async {
    var res = await PostApi.getPost(0);

    setState(() {
      post = res;
      isInit = true;
    });
  }

  @override
  void initState() {
    super.initState();

    WidgetsBinding.instance.addPostFrameCallback((_) {
      init();
    });
  }

  Widget load() {
    return Column(
      children: [
        UserNav(
          user: post?.user,
          goTo: goBack,
        ),
        const Expanded(
          child: PageLoading(),
        ),
      ],
    );
  }

  Widget content() {
    return Column(
      children: [
        UserNav(
          user: post?.user,
          goTo: goBack,
        ),
        Expanded(child: Container()),
      ],
    );
  }

  void goBack() {
    Get.back();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: SafeArea(
        child: Container(
          color: Colors.white,
          child: isInit ? content() : load(),
        ),
      ),
    );
  }
}
