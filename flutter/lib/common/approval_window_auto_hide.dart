import 'dart:async';

/// Never hide customer consent while another connection still needs a decision.
class ApprovalWindowAutoHide {
  final bool Function() hasPendingApproval;
  final void Function() minimize;
  Timer? _timer;

  ApprovalWindowAutoHide({
    required this.hasPendingApproval,
    required this.minimize,
  });

  void cancel() {
    _timer?.cancel();
    _timer = null;
  }

  void schedule() {
    cancel();
    if (hasPendingApproval()) return;
    _timer = Timer(const Duration(seconds: 3), () {
      _timer = null;
      // Another desktop or file request may arrive after scheduling.
      if (!hasPendingApproval()) minimize();
    });
  }
}
