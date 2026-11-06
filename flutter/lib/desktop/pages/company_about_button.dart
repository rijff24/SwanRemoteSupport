import 'package:flutter/material.dart';
import 'package:url_launcher/url_launcher.dart';

/// Available in both managed editions, including before technician sign-in.
class CompanyAboutButton extends StatelessWidget {
  final String companyName;
  final Future<String> Function() loadVersion;

  const CompanyAboutButton({
    super.key,
    required this.companyName,
    required this.loadVersion,
  });

  Future<void> _openLink(BuildContext context, String address) async {
    var opened = false;
    try {
      opened = await launchUrl(Uri.parse(address));
    } catch (_) {}
    if (!opened && context.mounted) {
      ScaffoldMessenger.of(context).showSnackBar(const SnackBar(
        content: Text('Could not open the browser. You can copy the address from this dialog.'),
      ));
    }
  }

  Widget _link(BuildContext context, String label, String address) => Column(
    crossAxisAlignment: CrossAxisAlignment.start,
    children: [
      TextButton(onPressed: () => _openLink(context, address), child: Text(label)),
      SelectableText(address),
    ],
  );

  Future<void> _showAbout(BuildContext context) async {
    var version = 'Version unavailable';
    try {
      final value = await loadVersion();
      if (value.trim().isNotEmpty) version = value;
    } catch (_) {}
    if (!context.mounted) return;
    final name = companyName.trim().isEmpty ? 'Swan Remote Support' : companyName;
    await showDialog<void>(
      context: context,
      builder: (dialogContext) => AlertDialog(
        title: Text('About $name'),
        scrollable: true,
        content: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            SelectableText('Swan Remote Support · $version'),
            const SizedBox(height: 12),
            const Text('Open-source remote support based on RustDesk. '
                'RustDesk copyright © Purslane Tech Pte. Ltd. and contributors.'),
            const SizedBox(height: 12),
            const Text('GNU Affero General Public License v3 (AGPL-3.0). '
                'Third-party components retain their own licences.'),
            _link(dialogContext, 'Project source', 'https://github.com/rijff24/SwanRemoteSupport'),
            _link(dialogContext, 'Read the AGPL licence', 'https://www.gnu.org/licenses/agpl-3.0.html'),
            _link(dialogContext, 'RustDesk source and notices', 'https://github.com/rustdesk/rustdesk'),
          ],
        ),
        actions: [
          TextButton(onPressed: () => Navigator.of(dialogContext).pop(), child: const Text('Close')),
        ],
      ),
    );
  }

  @override
  Widget build(BuildContext context) => TextButton.icon(
    onPressed: () => _showAbout(context),
    icon: const Icon(Icons.info_outline),
    label: const Text('About'),
  );
}
