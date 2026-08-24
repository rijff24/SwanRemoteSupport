import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'dart:math';

import 'package:file_picker/file_picker.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_hbb/common.dart';
import 'package:flutter_hbb/desktop/widgets/tabbar_widget.dart';
import 'package:flutter_hbb/models/platform_model.dart';
import 'package:flutter_hbb/models/state_model.dart';
import 'package:get/get.dart';
import 'package:path/path.dart';
import 'package:url_launcher/url_launcher_string.dart';
import 'package:window_manager/window_manager.dart';

const _swanPrivacyUrl =
    'https://github.com/rijff24/SwanRemoteSupport/blob/main/docs/PRIVACY.md';

class InstallPage extends StatefulWidget {
  const InstallPage({Key? key}) : super(key: key);

  @override
  State<InstallPage> createState() => _InstallPageState();
}

class _InstallPageState extends State<InstallPage> {
  final tabController = DesktopTabController(tabType: DesktopTabType.main);

  _InstallPageState() {
    Get.put<DesktopTabController>(tabController);
    const label = "install";
    tabController.add(TabInfo(
        key: label,
        label: label,
        closable: false,
        page: _InstallPageBody(
          key: const ValueKey(label),
        )));
  }

  @override
  void dispose() {
    super.dispose();
    Get.delete<DesktopTabController>();
  }

  @override
  Widget build(BuildContext context) {
    return DragToResizeArea(
      resizeEdgeSize: stateGlobal.resizeEdgeSize.value,
      enableResizeEdges: windowManagerEnableResizeEdges,
      child: Container(
        child: Scaffold(
            backgroundColor: Theme.of(context).colorScheme.background,
            body: DesktopTab(controller: tabController)),
      ),
    );
  }
}

class _InstallPageBody extends StatefulWidget {
  const _InstallPageBody({Key? key}) : super(key: key);

  @override
  State<_InstallPageBody> createState() => _InstallPageBodyState();
}

class _InstallPageBodyState extends State<_InstallPageBody>
    with WindowListener {
  static const _swanClipboardRetention = Duration(seconds: 90);

  late final TextEditingController controller;
  late final bool isSwanExpressInstall;
  late final TextEditingController swanPasswordController;
  Timer? _swanClipboardClearTimer;
  String? _swanCredentialReceipt;
  final RxBool startmenu = true.obs;
  final RxBool desktopicon = true.obs;
  final RxBool printer = false.obs;
  final RxBool swanConsentConfirmed = false.obs;
  final RxBool swanCredentialsSaved = false.obs;
  final RxString swanSetupError = ''.obs;
  final RxBool showProgress = false.obs;
  final RxBool btnEnabled = true.obs;

  // todo move to theme.
  final buttonStyle = OutlinedButton.styleFrom(
    textStyle: TextStyle(fontSize: 14, fontWeight: FontWeight.normal),
    padding: EdgeInsets.symmetric(vertical: 15, horizontal: 12),
  );

  _InstallPageBodyState() {
    controller = TextEditingController(text: bind.installInstallPath());
    isSwanExpressInstall = bind.mainGetAppNameSync() == 'Swan Remote Support';
    swanPasswordController =
        TextEditingController(text: _generateSwanPassword(24));
    final installOptions = jsonDecode(bind.installInstallOptions());
    startmenu.value = installOptions['STARTMENUSHORTCUTS'] != '0';
    desktopicon.value = installOptions['DESKTOPSHORTCUTS'] != '0';
    printer.value = installOptions['PRINTER'] == '1';
  }

  @override
  void initState() {
    windowManager.addListener(this);
    super.initState();
  }

  @override
  void dispose() {
    _swanClipboardClearTimer?.cancel();
    controller.dispose();
    swanPasswordController.dispose();
    windowManager.removeListener(this);
    super.dispose();
  }

  static String _generateSwanPassword(int length) {
    const alphabet =
        'ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz23456789!@#%+=_-';
    final random = Random.secure();
    return List.generate(
        length, (_) => alphabet[random.nextInt(alphabet.length)]).join();
  }

  Future<bool> _clearSwanCredentialReceiptFromClipboard() async {
    final receipt = _swanCredentialReceipt;
    if (receipt == null) {
      return true;
    }

    try {
      final clipboardData = await Clipboard.getData(Clipboard.kTextPlain);
      if (clipboardData?.text == receipt) {
        await Clipboard.setData(const ClipboardData(text: ''));
      }
    } on PlatformException {
      swanSetupError.value =
          'Windows could not clear the Swan credential receipt. Clear the clipboard manually after saving it.';
      return false;
    }

    if (_swanCredentialReceipt == receipt) {
      _swanCredentialReceipt = null;
    }
    _swanClipboardClearTimer?.cancel();
    _swanClipboardClearTimer = null;
    return true;
  }

  @override
  void onWindowClose() {
    gFFI.close();
    super.onWindowClose();
    windowManager.setPreventClose(false);
    windowManager.close();
  }

  InkWell Option(RxBool option, {String label = ''}) {
    return InkWell(
      // todo mouseCursor: "SystemMouseCursors.forbidden" or no cursor on btnEnabled == false
      borderRadius: BorderRadius.circular(6),
      onTap: () => btnEnabled.value ? option.value = !option.value : null,
      child: Row(
        children: [
          Obx(
            () => Checkbox(
              visualDensity: VisualDensity(horizontal: -4, vertical: -4),
              value: option.value,
              onChanged: (v) =>
                  btnEnabled.value ? option.value = !option.value : null,
            ).marginOnly(right: 8),
          ),
          Expanded(
            child: Text(translate(label)),
          ),
        ],
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final double em = 13;
    final isDarkTheme = MyTheme.currentThemeMode() == ThemeMode.dark;
    return Scaffold(
        backgroundColor: null,
        body: SingleChildScrollView(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                  isSwanExpressInstall
                      ? 'Swan Express Installation'
                      : translate('Installation'),
                  style: Theme.of(context).textTheme.headlineMedium),
              Row(
                children: [
                  Text('${translate('Installation Path')}:')
                      .marginOnly(right: 10),
                  Expanded(
                    child: TextField(
                      controller: controller,
                      readOnly: true,
                      decoration: InputDecoration(
                        contentPadding: EdgeInsets.all(0.75 * em),
                      ),
                    ).workaroundFreezeLinuxMint().marginOnly(right: 10),
                  ),
                  Obx(
                    () => OutlinedButton.icon(
                      icon: Icon(Icons.folder_outlined, size: 16),
                      onPressed: btnEnabled.value ? selectInstallPath : null,
                      style: buttonStyle,
                      label: Text(translate('Change Path')),
                    ),
                  )
                ],
              ).marginSymmetric(vertical: 2 * em),
              Option(startmenu, label: 'Create start menu shortcuts')
                  .marginOnly(bottom: 7),
              Option(desktopicon, label: 'Create desktop icon')
                  .marginOnly(bottom: 7),
              if (!isSwanExpressInstall)
                Option(printer, label: 'Install {$appName} Printer'),
              if (isSwanExpressInstall) _buildSwanCredentialsCard(context),
              Container(
                  padding: EdgeInsets.all(12),
                  decoration: BoxDecoration(
                    color: isDarkTheme
                        ? Color.fromARGB(135, 87, 87, 90)
                        : Colors.grey[100],
                    borderRadius: BorderRadius.circular(8),
                    border: Border.all(color: Colors.grey),
                  ),
                  child: Row(
                    children: [
                      Icon(Icons.info_outline_rounded, size: 32)
                          .marginOnly(right: 16),
                      Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Text(isSwanExpressInstall
                                  ? 'By installing, the computer owner authorizes the disclosed Swan support service and acknowledges the privacy information.'
                                  : translate('agreement_tip'))
                              .marginOnly(bottom: em),
                          InkWell(
                            hoverColor: Colors.transparent,
                            onTap: () => launchUrlString(
                                isSwanExpressInstall
                                    ? _swanPrivacyUrl
                                    : 'https://rustdesk.com/privacy.html'),
                            child: Tooltip(
                              message: isSwanExpressInstall
                                  ? _swanPrivacyUrl
                                  : 'https://rustdesk.com/privacy.html',
                              child: Row(children: [
                                Icon(Icons.launch_outlined, size: 16)
                                    .marginOnly(right: 5),
                                Text(
                                  isSwanExpressInstall
                                      ? 'Swan privacy information'
                                      : translate(
                                          'End-user license agreement'),
                                  style: const TextStyle(
                                      decoration: TextDecoration.underline),
                                )
                              ]),
                            ),
                          ),
                        ],
                      )
                    ],
                  )).marginSymmetric(vertical: 2 * em),
              Row(
                children: [
                  Expanded(
                    // NOT use Offstage to wrap LinearProgressIndicator
                    child: Obx(() => showProgress.value
                        ? LinearProgressIndicator().marginOnly(right: 10)
                        : Offstage()),
                  ),
                  Obx(
                    () => OutlinedButton.icon(
                      icon: Icon(Icons.close_rounded, size: 16),
                      label: Text(translate('Cancel')),
                      onPressed:
                          btnEnabled.value ? () => windowManager.close() : null,
                      style: buttonStyle,
                    ).marginOnly(right: 10),
                  ),
                  Obx(
                    () => ElevatedButton.icon(
                      icon: Icon(Icons.done_rounded, size: 16),
                      label: Text(isSwanExpressInstall
                          ? 'Express Install'
                          : translate('Accept and Install')),
                      onPressed: btnEnabled.value ? install : null,
                      style: buttonStyle,
                    ),
                  ),
                  Offstage(
                    offstage: bind.installShowRunWithoutInstall(),
                    child: Obx(
                      () => OutlinedButton.icon(
                        icon: Icon(Icons.screen_share_outlined, size: 16),
                        label: Text(translate('Run without install')),
                        onPressed: btnEnabled.value
                            ? () => bind.installRunWithoutInstall()
                            : null,
                        style: buttonStyle,
                      ).marginOnly(left: 10),
                    ),
                  ),
                ],
              )
            ],
          ).paddingSymmetric(horizontal: 4 * em, vertical: 3 * em),
        ));
  }

  Widget _buildSwanCredentialsCard(BuildContext context) {
    final serverId = gFFI.serverModel.serverId;
    return Container(
      padding: const EdgeInsets.all(14),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.primary.withOpacity(0.06),
        borderRadius: BorderRadius.circular(8),
        border: Border.all(
            color: Theme.of(context).colorScheme.primary.withOpacity(0.35)),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text('Private Swan support setup',
              style: Theme.of(context).textTheme.titleMedium),
          const SizedBox(height: 6),
          Text(
            'Tailscale must be connected. This installs a background service for password-protected unattended support. The computer owner can stop or uninstall it. Save the unique details below in the Swan password manager before installing; they will be hidden from the ordinary customer screen afterward.',
            style: Theme.of(context).textTheme.bodySmall,
          ),
          const SizedBox(height: 8),
          Obx(() => CheckboxListTile(
                contentPadding: EdgeInsets.zero,
                controlAffinity: ListTileControlAffinity.leading,
                value: swanConsentConfirmed.value,
                onChanged: (value) =>
                    swanConsentConfirmed.value = value ?? false,
                title: const Text(
                  'The computer owner has authorized unattended Swan support and has been shown how to stop or uninstall it.',
                ),
              )),
          const SizedBox(height: 12),
          TextField(
            controller: serverId,
            readOnly: true,
            decoration: const InputDecoration(
              labelText: 'Device ID',
              prefixIcon: Icon(Icons.computer),
            ),
          ).workaroundFreezeLinuxMint(),
          const SizedBox(height: 10),
          TextField(
            controller: swanPasswordController,
            readOnly: true,
            decoration: const InputDecoration(
              labelText: 'Unique unattended password',
              prefixIcon: Icon(Icons.key),
            ),
          ).workaroundFreezeLinuxMint(),
          const SizedBox(height: 10),
          Row(
            children: [
              OutlinedButton.icon(
                icon: const Icon(Icons.copy, size: 16),
                label: const Text('Copy credentials'),
                onPressed: () async {
                  final id = serverId.text.replaceAll(' ', '').trim();
                  if (!RegExp(r'^\d+$').hasMatch(id)) {
                    swanSetupError.value =
                        'Waiting for a device ID. Confirm Tailscale is connected and the Swan server is online.';
                    return;
                  }
                  final receipt =
                      'Customer computer: ${Platform.localHostname}\n'
                      'Swan Remote Support ID: $id\n'
                      'Unique password: ${swanPasswordController.text}';
                  try {
                    await Clipboard.setData(ClipboardData(text: receipt));
                  } on PlatformException {
                    swanSetupError.value =
                        'Windows could not copy the credential receipt. Installation was not unlocked.';
                    return;
                  }
                  _swanCredentialReceipt = receipt;
                  _swanClipboardClearTimer?.cancel();
                  _swanClipboardClearTimer = Timer(
                      _swanClipboardRetention,
                      () => unawaited(
                          _clearSwanCredentialReceiptFromClipboard()));
                  swanCredentialsSaved.value = true;
                  swanSetupError.value = '';
                },
              ),
              const SizedBox(width: 10),
              Expanded(
                child: Obx(() => Text(
                      swanCredentialsSaved.value
                          ? 'Copied. Paste into the password manager now. It clears after 90 seconds or when installation starts.'
                          : 'Installation remains locked until these details are copied.',
                      style: TextStyle(
                        color: swanCredentialsSaved.value
                            ? const Color(0xFF0A7D5A)
                            : Colors.orange,
                        fontWeight: FontWeight.w500,
                      ),
                    )),
              ),
            ],
          ),
          Obx(() => swanSetupError.value.isEmpty
              ? const SizedBox.shrink()
              : Padding(
                  padding: const EdgeInsets.only(top: 8),
                  child: Text(swanSetupError.value,
                      style: const TextStyle(color: Colors.red)),
                )),
        ],
      ),
    ).marginOnly(top: 12);
  }

  void install() async {
    if (isSwanExpressInstall) {
      if (!swanConsentConfirmed.value) {
        swanSetupError.value =
            'Confirm the computer owner\'s authorization before installing.';
        return;
      }
      final id =
          gFFI.serverModel.serverId.text.replaceAll(' ', '').trim();
      if (!RegExp(r'^\d+$').hasMatch(id)) {
        swanSetupError.value =
            'A valid device ID has not been received. Check Tailscale and the Swan server, then try again.';
        return;
      }
      if (!swanCredentialsSaved.value) {
        swanSetupError.value =
            'Copy and save this computer\'s unique credentials before installing.';
        return;
      }
      final passwordSaved = await bind.mainSetPermanentPasswordWithResult(
          password: swanPasswordController.text);
      if (!passwordSaved) {
        swanSetupError.value =
            'The unique unattended password could not be saved. Installation was not started.';
        return;
      }
      await bind.mainSetOption(key: 'approve-mode', value: 'password');
      await bind.mainSetOption(
          key: 'verification-method', value: 'use-permanent-password');
      await bind.mainSetOption(key: 'allow-only-conn-window-open', value: 'N');
      final clipboardCleared =
          await _clearSwanCredentialReceiptFromClipboard();
      if (!clipboardCleared) {
        return;
      }
    }

    do_install() {
      btnEnabled.value = false;
      showProgress.value = true;
      String args = '';
      if (startmenu.value) args += ' startmenu';
      if (desktopicon.value) args += ' desktopicon';
      if (!isSwanExpressInstall && printer.value) args += ' printer';
      bind.installInstallMe(options: args, path: controller.text);
    }

    do_install();
  }

  void selectInstallPath() async {
    String? install_path = await FilePicker.platform
        .getDirectoryPath(initialDirectory: controller.text);
    if (install_path != null) {
      controller.text = join(install_path, await bind.mainGetAppName());
    }
  }
}
