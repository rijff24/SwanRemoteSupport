import 'dart:convert';
import 'dart:async';
import 'dart:io';
import 'package:flutter/material.dart';
import 'package:flutter_svg/flutter_svg.dart';
import 'package:flutter_hbb/common.dart';
import 'package:flutter_hbb/models/platform_model.dart';
import 'company_contact_links.dart';

/// Company authorization lives in Rust and on the receiver. This view never
/// receives a bearer token, signing key or reusable device credential.
class CompanyTechnicianPage extends StatefulWidget {
  const CompanyTechnicianPage({super.key});
  @override
  State<CompanyTechnicianPage> createState() => _CompanyTechnicianPageState();
}

class _CompanyTechnicianPageState extends State<CompanyTechnicianPage> {
  final _username = TextEditingController();
  final _password = TextEditingController();
  final _code = TextEditingController();
  Map<String, dynamic> _company = {};
  List<dynamic> _devices = [];
  List<dynamic> _history = [];
  bool _loggedIn = false;
  bool _busy = false;
  String _error = '';
  String _updateError = '';
  bool _recoveryPending = false;
  String? _approvedUpdate;
  Timer? _refreshTimer;
  Timer? _updateTimer;
  bool _pollingUpdate = false;
  bool _updateRunning = false;

  Future<dynamic> _request(Map<String, dynamic> input) async {
    final text = await bind.mainCompanyRequest(request: jsonEncode(input))
        .first.timeout(input['action'] == 'resume-update'
            ? const Duration(minutes: 25) : const Duration(seconds: 25));
    final response = jsonDecode(text) as Map<String, dynamic>;
    if (response['ok'] != true) {
      throw Exception(response['error'] ?? 'Company request failed');
    }
    return response['data'];
  }

  Future<void> _run(Future<void> Function() action) async {
    if (_busy) return;
    setState(() { _busy = true; _error = ''; });
    try { await action(); }
    catch (error) { if (mounted) setState(() { _error = error.toString(); }); }
    finally { if (mounted) setState(() { _busy = false; }); }
  }

  Future<void> _pollUpdate() async {
    if (_pollingUpdate || !mounted || !Platform.isWindows) return;
    _pollingUpdate = true;
    try {
      final progress = await _request({'action': 'update-progress'}) as Map<String, dynamic>;
      if (progress['handed_off'] == true) exit(0);
      if (mounted) setState(() {
        _updateRunning = progress['running'] == true;
        if (progress['failed'] == true && !_recoveryPending) {
          _updateError = 'Software update was deferred or could not finish. It will retry automatically.';
        }
      });
    } catch (_) {
      if (mounted) setState(() { _updateError = 'Software update status is unavailable.'; });
    } finally { _pollingUpdate = false; }
  }

  Future<void> _refresh() async {
    final cachedCompany = jsonDecode(await bind.mainGetCommon(key: 'company-overview'))
        as Map<String, dynamic>;
    if (mounted) setState(() { _company = cachedCompany; });
    if (Platform.isWindows) {
      try {
        final recovery = await _request({'action': 'resume-update'}) as Map<String, dynamic>;
        if (recovery['handed_off'] == true) exit(0);
        if (mounted) setState(() {
          _recoveryPending = recovery['pending'] == true;
          _updateError = _recoveryPending ? 'Software installation recovery is pending. New support connections are unavailable.' : '';
        });
      } catch (_) {
        if (mounted) setState(() {
          _recoveryPending = true;
          _updateError = 'Software recovery could not finish. It will retry automatically.';
        });
      }
    }
    // Retain cached branding if the server is offline; authenticated operations
    // still require a current signed policy on the native side.
    try { await _request({'action': 'sync'}); } catch (_) {}
    final company = jsonDecode(await bind.mainGetCommon(key: 'company-overview'))
        as Map<String, dynamic>;
    final status = await _request({'action': 'status'}) as Map<String, dynamic>;
    final loggedIn = status['logged_in'] == true;
    if (mounted) setState(() {
      _company = company; _loggedIn = loggedIn;
      if (!loggedIn) { _devices = []; _history = []; _approvedUpdate = null; }
    });
    final devices = loggedIn ? await _request({'action': 'devices'}) as List<dynamic> : <dynamic>[];
    final history = loggedIn ? await _request({'action': 'history'}) as List<dynamic> : <dynamic>[];
    if (mounted) setState(() { _devices = devices; _history = history; });
    try {
    if (mounted && loggedIn && Platform.isWindows && !_recoveryPending) {
      final result = await _request({'action': 'update'}) as Map<String, dynamic>;
      // Rust writes the signed recovery receipt and starts the verified helper
      // before acknowledging handoff. New sessions are then blocked natively.
      // Run this even without new metadata so an interrupted handoff can retry.
      if (result['handed_off'] == true) exit(0);
    }
    final update = loggedIn ? await _request({'action': 'update-status'}) : null;
    if (mounted) setState(() {
      _company = company; _loggedIn = loggedIn; _devices = devices; _history = history;
      _approvedUpdate = update is Map ? update['version'] as String? : null;
    });
    } catch (_) {
      if (mounted) setState(() {
        _approvedUpdate = null;
        if (!_recoveryPending) _updateError = 'Software update could not finish. It will retry automatically.';
      });
    }
  }

  Future<void> _login() async {
    final request = {'action': 'login', 'username': _username.text.trim(),
      'password': _password.text, 'code': _code.text.trim()};
    _password.clear(); _code.clear();
    await _request(request);
    await _refresh();
  }

  Future<void> _logout() async {
    // Clear display state even when the server is offline. Rust clears its
    // token and pending tickets before attempting server logout.
    try { await _request({'action': 'logout'}); }
    finally { if (mounted) setState(() { _loggedIn = false; _devices = []; _history = []; }); }
  }

  Future<void> _connect(Map<String, dynamic> device, {bool unattended = false}) async {
    final response = await _request({'action': 'connect', 'device_id': device['id'],
      'unattended': unattended}) as Map<String, dynamic>;
    if (!mounted) return;
    // Only a one-use local handle enters the window API; signed grants and
    // challenge proof keys stay in Rust memory and are bound to the target.
    await connect(context, response['rustdesk_id'] as String,
        password: response['ticket_handle'] as String);
  }

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) { if (mounted) _run(_refresh); });
    _refreshTimer = Timer.periodic(const Duration(seconds: 60), (_) {
      if (mounted && !_busy) _run(_refresh);
    });
    _updateTimer = Timer.periodic(const Duration(seconds: 1), (_) { _pollUpdate(); });
  }

  @override
  void dispose() {
    _refreshTimer?.cancel();
    _updateTimer?.cancel();
    _username.dispose(); _password.dispose(); _code.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final color = _company['primary_color'] as String? ?? '#007F82';
    final brandColor = RegExp(r'^#[0-9a-fA-F]{6}$').hasMatch(color)
        ? Color(0xFF000000 | int.parse(color.substring(1), radix: 16)) : const Color(0xFF007F82);
    return SingleChildScrollView(
      padding: const EdgeInsets.all(24),
      child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
        if ((_company['logo_svg'] as String? ?? '').isNotEmpty)
          SvgPicture.string(_company['logo_svg'] as String, height: 64, width: 180),
        const SizedBox(height: 12),
        Text(_company['display_name'] as String? ?? 'Swan Remote Support Technician',
            style: Theme.of(context).textTheme.headlineSmall?.copyWith(color: brandColor)),
        Text(_company['domain'] as String? ?? 'Company setup required'),
        CompanyContactLinks(company: _company),
        if (_approvedUpdate != null) Text('Company-approved update available: $_approvedUpdate'),
        if (_updateError.isNotEmpty) Text(_updateError),
        if (_updateRunning) const Text('Preparing company-approved software update…'),
        const SizedBox(height: 16),
        if (_busy) const LinearProgressIndicator(),
        if (_error.isNotEmpty) Padding(padding: const EdgeInsets.symmetric(vertical: 12),
            child: Text(_error, style: const TextStyle(color: Colors.red))),
        if (!_loggedIn) SizedBox(width: 360, child: Column(children: [
          TextField(controller: _username, decoration: const InputDecoration(labelText: 'Technician username')),
          TextField(controller: _password, obscureText: true, enableSuggestions: false,
              autocorrect: false, decoration: const InputDecoration(labelText: 'Password')),
          TextField(controller: _code, keyboardType: TextInputType.number, maxLength: 6,
              decoration: const InputDecoration(labelText: 'Authenticator code'),
              onSubmitted: (_) { if (!_busy) _run(_login); }),
          const SizedBox(height: 12),
          ElevatedButton(onPressed: _busy ? null : () => _run(_login), child: const Text('Sign in')),
        ])),
        if (_loggedIn) ...[
          Row(children: [
            ElevatedButton.icon(onPressed: _busy ? null : () => _run(_refresh),
                icon: const Icon(Icons.refresh), label: const Text('Refresh')),
            const SizedBox(width: 12),
            TextButton(onPressed: _busy ? null : () => _run(_logout), child: const Text('Sign out')),
          ]),
          const SizedBox(height: 16),
          Text('Authorized devices', style: Theme.of(context).textTheme.titleLarge),
          if (_devices.isEmpty) const Padding(padding: EdgeInsets.symmetric(vertical: 12),
              child: Text('No approved devices are assigned to your account.')),
          for (final value in _devices) Builder(builder: (context) {
            final device = value as Map<String, dynamic>;
            final approved = device['state'] == 'approved';
            return Card(child: ListTile(title: Text(device['name'] as String),
              subtitle: Text('${device['group']} · ${device['state']}'),
              trailing: Wrap(spacing: 8, children: [
                TextButton(onPressed: !_busy && approved && !_recoveryPending ? () => _run(() => _connect(device)) : null,
                    child: const Text('Request support')),
                if (device['unattended'] == true)
                  TextButton(onPressed: !_busy && approved && !_recoveryPending ? () => _run(() => _connect(device, unattended: true)) : null,
                      child: const Text('Unattended')),
              ])));
          }),
          const SizedBox(height: 20),
          Text('Session history', style: Theme.of(context).textTheme.titleLarge),
          if (_history.isEmpty) const Text('No sessions yet.'),
          for (final value in _history) Builder(builder: (context) {
            final session = value as Map<String, dynamic>;
            final time = DateTime.fromMillisecondsSinceEpoch((session['requested_at'] as int) * 1000).toLocal();
            final active = session['claimed'] == true && session['closed'] != true &&
                (session['lease_until'] as int) * 1000 > DateTime.now().millisecondsSinceEpoch;
            return ListTile(title: Text(session['device_name'] as String),
              subtitle: Text('$time · ${session['unattended'] == true ? 'Unattended' : 'Customer approval'}'),
              trailing: Text(active ? 'Active' : session['claimed'] == true ? 'Ended' : 'Requested'));
          }),
        ],
        const SizedBox(height: 24),
        const Text('Powered by Swan Remote Support and RustDesk · AGPL-3.0'),
      ]),
    );
  }
}
