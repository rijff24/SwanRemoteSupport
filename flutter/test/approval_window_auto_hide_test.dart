import 'package:flutter_hbb/common/approval_window_auto_hide.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  testWidgets('desktop approval keeps a pending file consent visible',
      (tester) async {
    var pendingFileApproval = true;
    var minimizations = 0;
    final autoHide = ApprovalWindowAutoHide(
      hasPendingApproval: () => pendingFileApproval,
      minimize: () => minimizations++,
    );
    addTearDown(autoHide.cancel);

    autoHide.schedule(); // Desktop approved; file request still pending.
    await tester.pump(const Duration(seconds: 4));
    expect(minimizations, 0);

    pendingFileApproval = false;
    autoHide.schedule();
    await tester.pump(const Duration(seconds: 3));
    expect(minimizations, 1);
  });

  testWidgets('request arriving during the hide delay prevents minimizing',
      (tester) async {
    var pending = false;
    var minimizations = 0;
    final autoHide = ApprovalWindowAutoHide(
      hasPendingApproval: () => pending,
      minimize: () => minimizations++,
    );
    addTearDown(autoHide.cancel);

    autoHide.schedule();
    await tester.pump(const Duration(seconds: 2));
    pending = true;
    await tester.pump(const Duration(seconds: 2));
    expect(minimizations, 0);

    pending = false;
    autoHide.schedule();
    await tester.pump(const Duration(seconds: 1));
    autoHide.cancel(); // New pending request cancels the earlier callback.
    await tester.pump(const Duration(seconds: 3));
    expect(minimizations, 0);
  });
}
