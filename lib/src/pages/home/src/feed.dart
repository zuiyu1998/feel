import 'dart:async';

import 'package:feel/src/apis/feed/index.dart';
import 'package:feel/src/apis/feed/model.dart';
import 'package:feel/src/apis/post/model.dart';
import 'package:feel/src/router/index.dart';
import 'package:feel/src/utils/date_time.dart';
import 'package:flutter/material.dart';
import 'package:get/get.dart';

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
      padding: const EdgeInsets.fromLTRB(8, 12, 0, 0),
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

  void goToDetail() {
    switch (item.source.kind) {
      case FeedKind.post:
        Get.toNamed(Routes.post, arguments: {"id": item.source.id});
        break;
      default:
    }
  }

  Widget feedSource() {
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
                children: [Text(fromNow(item.createAt))],
              )
            ],
          ),
        ))
      ],
    );
  }

  Widget feedSummary() {
    switch (item.source.kind) {
      case FeedKind.post:
        var postItem = PostSummary.fromJson(item.summary);
        return PostSummaryItem(item: postItem);

      default:
        return Container();
    }
  }

  Widget feedActions() {
    return Container(
      margin: const EdgeInsets.fromLTRB(0.0, 8, 0.0, 0.0),
      child: Row(
        children: [
          Expanded(
              child: Row(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              const Icon(
                Icons.share_outlined,
                size: 20,
              ),
              Container(
                margin: const EdgeInsets.fromLTRB(6.0, 0.0, 0.0, 0.0),
                child: Text(
                  "${item.share}",
                ),
              )
            ],
          )),
          Expanded(
              child: InkWell(
            onTap: () {
              goToDetail();
            },
            child: Row(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                const Icon(
                  Icons.chat_bubble_outline_rounded,
                  size: 20,
                ),
                Container(
                  margin: const EdgeInsets.fromLTRB(6.0, 0.0, 0.0, 0.0),
                  child: Text(
                    "${item.like}",
                  ),
                )
              ],
            ),
          )),
          Expanded(
              child: Row(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              const Icon(
                Icons.thumb_up_alt_outlined,
                size: 20,
              ),
              Container(
                margin: const EdgeInsets.fromLTRB(6.0, 0.0, 0.0, 0.0),
                child: Text(
                  "${item.comment}",
                ),
              )
            ],
          )),
        ],
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Container(
      color: Colors.white,
      padding: const EdgeInsets.all(8),
      margin: const EdgeInsets.fromLTRB(0.0, 0.0, 0.0, 16.0),
      child: Column(
        children: [feedSource(), feedSummary(), feedActions()],
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
