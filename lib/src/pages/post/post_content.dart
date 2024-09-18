import 'package:feel/src/apis/post/model.dart';
import 'package:feel/src/components/user_nav.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:get/get.dart';

typedef SliverHeaderBuilder = Widget Function(
    BuildContext context, double shrinkOffset, bool overlapsContent);

class SliverHeaderDelegate extends SliverPersistentHeaderDelegate {
  SliverHeaderDelegate({
    required this.maxHeight,
    this.minHeight = 0,
    required Widget child,
  })  : builder = ((a, b, c) => child),
        assert(minHeight <= maxHeight && minHeight >= 0);

  SliverHeaderDelegate.fixedHeight({
    required double height,
    required Widget child,
  })  : builder = ((a, b, c) => child),
        maxHeight = height,
        minHeight = height;

  SliverHeaderDelegate.builder({
    required this.maxHeight,
    this.minHeight = 0,
    required this.builder,
  });

  final double maxHeight;
  final double minHeight;
  final SliverHeaderBuilder builder;

  @override
  Widget build(
    BuildContext context,
    double shrinkOffset,
    bool overlapsContent,
  ) {
    Widget child = builder(context, shrinkOffset, overlapsContent);
    assert(() {
      if (child.key != null) {
        debugPrint(
            '${child.key}: shrink: $shrinkOffset,overlaps:$overlapsContent');
      }
      return true;
    }());

    return SizedBox.expand(child: child);
  }

  @override
  double get maxExtent => maxHeight;

  @override
  double get minExtent => minHeight;

  @override
  bool shouldRebuild(SliverHeaderDelegate oldDelegate) {
    return oldDelegate.maxExtent != maxExtent ||
        oldDelegate.minExtent != minExtent;
  }
}

class PostContent extends StatefulWidget {
  final Post post;

  const PostContent({super.key, required this.post});

  @override
  State<PostContent> createState() => _PostContentState();
}

class _PostContentState extends State<PostContent>
    with TickerProviderStateMixin {
  late TabController _tabController;
  final ScrollController _scrollController = ScrollController();

  double opacity = 0.0;

  double headerHeightMax = 48.0;

  Widget buildHeader() {
    return Container(
      color: Colors.white,
      child: UserNav(user: widget.post.user, goTo: goBack, opacity: opacity),
    );
  }

  Widget buildTabBar() {
    return Container(
      color: Colors.white,
      child: TabBar(
        controller: _tabController,
        tabs: const [
          Tab(text: '评论'),
          Tab(text: '赞'),
        ],
      ),
    );
  }

  void onScroll() {
    var headerHeight =
        clampDouble(_scrollController.offset, 0.0, headerHeightMax);
    var tmpOpacity = headerHeight / headerHeightMax;

    setState(() {
      opacity = tmpOpacity;
    });
  }

  @override
  void initState() {
    _tabController = TabController(length: 2, vsync: this);

    _scrollController.addListener(onScroll);

    super.initState();
  }

  @override
  void dispose() {
    _tabController.dispose();
    _scrollController.removeListener(onScroll);

    super.dispose();
  }

  void goBack() {
    Get.back();
  }

  @override
  Widget build(BuildContext context) {
    return NestedScrollView(
        controller: _scrollController,
        headerSliverBuilder: (context, innerBoxIsScrolled) {
          return [
            SliverPersistentHeader(
              pinned: true,
              delegate: SliverHeaderDelegate(
                //有最大和最小高度
                maxHeight: headerHeightMax,
                minHeight: headerHeightMax,
                child: buildHeader(),
              ),
            ),
            SliverToBoxAdapter(
              child: Container(
                height: 200,
                color: Colors.red,
              ),
            ),
            SliverPersistentHeader(
              pinned: true,
              delegate: SliverHeaderDelegate(
                //有最大和最小高度
                maxHeight: 48,
                minHeight: 48,
                child: buildTabBar(),
              ),
            ),
          ];
        },
        body: TabBarView(
          controller: _tabController,
          children: [
            // 第一个选项卡的内容
            ListView.builder(
              itemCount: 20,
              itemBuilder: (BuildContext context, int index) {
                return ListTile(title: Text('Item $index'));
              },
            ),
            // 第二个选项卡的内容
            ListView.builder(
              itemCount: 10,
              itemBuilder: (BuildContext context, int index) {
                return ListTile(title: Text('Item $index'));
              },
            ),
          ],
        ));
  }
}
