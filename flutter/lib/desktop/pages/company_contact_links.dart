import 'package:flutter/material.dart';
import 'package:url_launcher/url_launcher.dart';

/// Display only public, signed company information. Links never execute commands.
class CompanyContactLinks extends StatelessWidget {
  final Map<String, dynamic> company;
  const CompanyContactLinks({super.key, required this.company});

  Widget _link(String label, String address) => TextButton.icon(
    onPressed: () async {
      final url = Uri.tryParse(address);
      if (url != null && url.scheme == 'https' && url.host.isNotEmpty && url.userInfo.isEmpty) {
        await launchUrl(url);
      }
    },
    icon: const Icon(Icons.open_in_new),
    label: Text(label),
  );

  @override
  Widget build(BuildContext context) {
    final contacts = company['support_contacts'] as String? ?? '';
    final support = company['support_url'] as String? ?? '';
    final shortcuts = company['shortcuts'] is List ? company['shortcuts'] as List : const [];
    return Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
      if (contacts.isNotEmpty) SelectableText(contacts),
      if (support.isNotEmpty) _link('Contact support', support),
      for (final entry in shortcuts.whereType<Map>())
        if (entry['label'] is String && entry['url'] is String)
          _link(entry['label'] as String, entry['url'] as String),
    ]);
  }
}
