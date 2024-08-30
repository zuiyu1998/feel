import 'package:feel/src/apis/feed/index.dart';
import 'package:feel/src/apis/feed/model.dart';
import 'package:flutter/widgets.dart';

class FeedPage extends StatefulWidget {
  const FeedPage({
    super.key,
  });

  @override
  State<FeedPage> createState() => _FeedPageState();
}

class _FeedPageState extends State<FeedPage> {
  final List<Feed> data = [];

  Future<void> _getData() async {
    var res = await FeedApi.getFeedPagelist();
    debugPrint(res.data[0].toJson().toString());
  }

  @override
  void initState() {
    super.initState();

    WidgetsBinding.instance.addPostFrameCallback((_) {
      _getData();
    });
  }

  @override
  Widget build(BuildContext context) {
    return Container();
  }
}
