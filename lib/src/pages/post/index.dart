import 'package:dio/dio.dart';
import 'package:feel/src/apis/post/index.dart';
import 'package:feel/src/apis/post/model.dart';
import 'package:feel/src/components/loading/page_loading.dart';
import 'package:feel/src/pages/post/post_content.dart';
import 'package:flutter/material.dart';

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
    try {
      var res = await PostApi.getPost(0);

      setState(() {
        post = res;
        isInit = true;
      });
    } on DioException catch (_) {}
  }

  @override
  void initState() {
    super.initState();

    WidgetsBinding.instance.addPostFrameCallback((_) {
      init();
    });
  }

  Widget load() {
    return const PageLoading();
  }

  Widget content() {
    return Column(
      children: [Expanded(child: PostContent(post: post!))],
    );
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
