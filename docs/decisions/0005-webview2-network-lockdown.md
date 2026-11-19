# 0005. Lock down WebView2 background networking with browser args
- Date: 2026-11-18
- Status: accepted (user decision 2026-11-18; the specific flags are best-effort)
- Decision: Set `additionalBrowserArgs` in `tauri.conf.json`. It repeats wry's default `--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection`, adds `DnsOverHttps,AsyncDns`, and adds `--disable-background-networking --disable-component-update --disable-domain-reliability --disable-sync --disable-client-side-phishing-detection --no-pings --disable-quic --proxy-server=http://127.0.0.1:9 --proxy-bypass-list=localhost;*.localhost;127.0.0.1;[::1]`. Every release is verified by checking that no PID in the tree has a non-loopback TCP connection.
- Reason: The tester saw WebView2 children connect to 52.97.x.x, 8.8.8.8, and 8.8.4.4 on port 443. There is no documented WebView2 API to stop this, and Microsoft warns that flags may change, so a dead-loopback proxy acts as a catch-all.
