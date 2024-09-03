import 'dart:async';

import 'package:feel/src/apis/feed/index.dart';
import 'package:feel/src/apis/feed/model.dart';
import 'package:feel/src/apis/post/model.dart';
import 'package:flutter/material.dart';

class FeedPage extends StatefulWidget {
  const FeedPage({
    super.key,
  });

  @override
  State<FeedPage> createState() => _FeedPageState();
}

class PostSummaryItem extends StatelessWidget {
  final PostSummary item;

  const PostSummaryItem({super.key, required this.item});

  @override
  Widget build(BuildContext context) {
    List<Widget> list = [];

    if (item.content != null) {
      list.add(Text(
        item.content as String,
      ));
    }

    if (item.images != null) {}

    return Container(
      padding: const EdgeInsets.fromLTRB(0, 12, 0, 0),
      child: Row(
        children: [
          Column(
            children: list,
          )
        ],
      ),
    );
  }
}

class FeedItem extends StatelessWidget {
  final Feed item;

  const FeedItem({super.key, required this.item});

  Widget userTop() {
    return Row(
      children: [
        Image.network(
          item.source.user.avatar,
          width: 48,
          height: 48,
        ),
        Expanded(
            child: Container(
          padding: const EdgeInsets.fromLTRB(8, 0, 0, 0),
          child: Column(
            children: [
              Row(
                children: [Text(item.source.user.nikename)],
              ),
              Row(
                children: [Text(item.source.user.nikename)],
              )
            ],
          ),
        ))
      ],
    );
  }

  Widget feedKind() {
    switch (item.source.kind) {
      case FeedKind.post:
        var postItem = PostSummary.fromJson(item.summary);
        return PostSummaryItem(item: postItem);

      default:
        return Container();
    }
  }

  @override
  Widget build(BuildContext context) {
    return Container(
      color: Colors.white,
      padding: const EdgeInsets.all(8),
      child: Column(
        children: [userTop(), feedKind()],
      ),
    );
  }
}

class _FeedPageState extends State<FeedPage> {
  final StreamController<List<Feed>> _streamController = StreamController();

  Future<void> _getData() async {
    var res = await FeedApi.getFeedPagelist();
    _streamController.add(res.data);
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
    return RefreshIndicator(
      onRefresh: () {
        return Future.value();
      },
      child: StreamBuilder<List<Feed>>(
        stream: _streamController.stream,
        builder: (context, snapshot) {
          List<Feed>? data = snapshot.data;
          if (data == null) {
            return Container();
          }

          return ListView.builder(
            itemCount: data.length,
            itemBuilder: (context, index) => FeedItem(
              item: data[index],
            ),
          );
        },
      ),
    );
  }
}
