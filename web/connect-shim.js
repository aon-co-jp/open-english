// connect-shim.js — 旧REST風パス(/v1/...)の呼び出しを、Connectの単純呼び出し
// (POST /openenglish.v1.<Service>/<Method>、JSON本文)へ書き換える薄い層(2026-11-05新設)。
// サーバー側の旧RESTルートは削除済み。呼び出し側のコード(app.js等)は旧パスのまま書かれているため、
// fetch を一箇所で包んで新パスへ送る。同一オリジンは全て、別オリジンは公開API(/v1/public/・/v1/custom-qa)のみ書き換える
// (別オリジンの aruaru-llm 自身の /v1/... はそのまま)。元GETの引数はURLのクエリのまま送り、サーバーが受ける。
(function () {
  "use strict";
  var GET = {
 "/v1/admin/aruaru-llm/models": "/openenglish.v1.AdminAruaruLlm/Models",
 "/v1/admin/aruaru-llm/runtime": "/openenglish.v1.AdminAruaruLlm/Runtime",
 "/v1/public/aruaru-llm/recommend": "/openenglish.v1.AruaruLlm/Recommend",
 "/v1/public/news/ask": "/openenglish.v1.News/Ask",
 "/v1/public/news/stock": "/openenglish.v1.News/Stock",
 "/v1/internal/news/export": "/openenglish.v1.Internal/NewsExport",
 "/v1/public/aruaru-llm/runtime": "/openenglish.v1.AruaruLlm/Runtime",
 "/v1/public/aruaru-llm/accelerators": "/openenglish.v1.AruaruLlm/Accelerators",
 "/v1/public/aruaru-llm/models/catalog": "/openenglish.v1.AruaruLlm/ModelsCatalog",
 "/v1/public/aruaru-llm/switch-status": "/openenglish.v1.AruaruLlm/SwitchStatus",
 "/v1/public/chat-providers/active": "/openenglish.v1.ChatProviders/Active",
 "/v1/public/news/latest": "/openenglish.v1.News/Latest",
 "/v1/public/knowledge/export": "/openenglish.v1.Knowledge/Export",
 "/v1/public/news/for": "/openenglish.v1.News/For",
 "/v1/public/news/archive-search": "/openenglish.v1.News/ArchiveSearch",
 "/v1/public/tts/status": "/openenglish.v1.Tts/Status",
 "/v1/config": "/openenglish.v1.Config/Config",
 "/v1/platform-info": "/openenglish.v1.PlatformInfo/PlatformInfo",
 "/v1/cpu-runtime": "/openenglish.v1.CpuRuntime/CpuRuntime",
 "/v1/world-languages": "/openenglish.v1.WorldLanguages/WorldLanguages",
 "/v1/region-info": "/openenglish.v1.RegionInfo/RegionInfo",
 "/v1/region-news": "/openenglish.v1.RegionNews/RegionNews",
 "/v1/auth/config": "/openenglish.v1.Auth/ConfigGet",
 "/v1/auth/qr-login/status": "/openenglish.v1.Auth/QrLoginStatus",
 "/v1/auth/qr-login/whoami": "/openenglish.v1.Auth/QrLoginWhoami",
 "/v1/auth/session": "/openenglish.v1.Auth/Session",
 "/v1/db/history": "/openenglish.v1.Db/HistoryGet",
 "/v1/db/settings": "/openenglish.v1.Db/SettingsGet",
 "/v1/custom-qa": "/openenglish.v1.CustomQa/CustomQaGet",
 "/v1/db/info": "/openenglish.v1.Db/Info",
 "/v1/fs/list-dir": "/openenglish.v1.Fs/ListDir",
 "/v1/network/status": "/openenglish.v1.Network/Status",
 "/v1/updates/history": "/openenglish.v1.Updates/History",
 "/v1/agent/local/read": "/openenglish.v1.Agent/LocalRead",
 "/v1/agent/local/status": "/openenglish.v1.Agent/LocalStatus",
 "/v1/agent/vps/read": "/openenglish.v1.Agent/VpsRead",
 "/v1/agent/vps/status": "/openenglish.v1.Agent/VpsStatus",
 "/v1/agent/github/read": "/openenglish.v1.Agent/GithubRead",
 "/v1/agent/github/status": "/openenglish.v1.Agent/GithubStatus",
 "/v1/world-lab/status": "/openenglish.v1.WorldLab/Status",
 "/v1/world-lab/devices": "/openenglish.v1.WorldLab/Devices",
 "/v1/world-lab/dispatch/pending": "/openenglish.v1.WorldLab/DispatchPending"
};
  var POST = {
 "/v1/admin/aruaru-llm/recommend-and-download": "/openenglish.v1.AdminAruaruLlm/RecommendAndDownload",
 "/v1/admin/aruaru-llm/models/select": "/openenglish.v1.AdminAruaruLlm/ModelsSelect",
 "/v1/admin/aruaru-llm/models/install": "/openenglish.v1.AdminAruaruLlm/ModelsInstall",
 "/v1/admin/aruaru-llm/download-larger": "/openenglish.v1.AdminAruaruLlm/DownloadLarger",
 "/v1/admin/aruaru-llm/download-smaller": "/openenglish.v1.AdminAruaruLlm/DownloadSmaller",
 "/v1/admin/aruaru-llm/qwen/select": "/openenglish.v1.AdminAruaruLlm/QwenSelect",
 "/v1/admin/aruaru-llm/qwen/install": "/openenglish.v1.AdminAruaruLlm/QwenInstall",
 "/v1/public/aruaru-llm/models/select": "/openenglish.v1.AruaruLlm/ModelsSelect",
 "/v1/public/aruaru-llm/generate": "/openenglish.v1.AruaruLlm/Generate",
 "/v1/public/aruaru-llm/generate-with-search": "/openenglish.v1.AruaruLlm/GenerateWithSearch",
 "/v1/public/chat-providers/complete-priority": "/openenglish.v1.ChatProviders/CompletePriority",
 "/v1/public/persona/prompt": "/openenglish.v1.Persona/Prompt",
 "/v1/public/freelance/job-search": "/openenglish.v1.Freelance/JobSearch",
 "/v1/public/tts": "/openenglish.v1.Tts/Tts",
 "/v1/auth/config": "/openenglish.v1.Auth/ConfigSet",
 "/v1/auth/request-otp": "/openenglish.v1.Auth/RequestOtp",
 "/v1/auth/request-sms-otp": "/openenglish.v1.Auth/RequestSmsOtp",
 "/v1/auth/verify-otp": "/openenglish.v1.Auth/VerifyOtp",
 "/v1/auth/qr-login/start": "/openenglish.v1.Auth/QrLoginStart",
 "/v1/auth/qr-login/confirm": "/openenglish.v1.Auth/QrLoginConfirm",
 "/v1/auth/qr-login/finish": "/openenglish.v1.Auth/QrLoginFinish",
 "/v1/auth/logout": "/openenglish.v1.Auth/Logout",
 "/v1/auth/totp-setup": "/openenglish.v1.Auth/TotpSetup",
 "/v1/auth/totp-verify": "/openenglish.v1.Auth/TotpVerify",
 "/v1/db/history": "/openenglish.v1.Db/HistorySet",
 "/v1/db/history/clear": "/openenglish.v1.Db/HistoryClear",
 "/v1/db/settings": "/openenglish.v1.Db/SettingsSet",
 "/v1/custom-qa": "/openenglish.v1.CustomQa/CustomQaSet",
 "/v1/db/storage-path": "/openenglish.v1.Db/StoragePath",
 "/v1/duckdns/update": "/openenglish.v1.Duckdns/Update",
 "/v1/db/rsync-backup": "/openenglish.v1.Db/RsyncBackup",
 "/v1/db/rsync-backup-all": "/openenglish.v1.Db/RsyncBackupAll",
 "/v1/db/migrate-legacy": "/openenglish.v1.Db/MigrateLegacy",
 "/v1/db/install-rsync": "/openenglish.v1.Db/InstallRsync",
 "/v1/updates/downgrade": "/openenglish.v1.Updates/Downgrade",
 "/v1/agent/local/write": "/openenglish.v1.Agent/LocalWrite",
 "/v1/agent/vps/write": "/openenglish.v1.Agent/VpsWrite",
 "/v1/agent/github/commit": "/openenglish.v1.Agent/GithubCommit",
 "/v1/agent/github/create-and-push": "/openenglish.v1.Agent/GithubCreateAndPush",
 "/v1/world-lab/pair": "/openenglish.v1.WorldLab/Pair",
 "/v1/world-lab/unpair": "/openenglish.v1.WorldLab/Unpair",
 "/v1/world-lab/pair/bulk": "/openenglish.v1.WorldLab/PairBulk",
 "/v1/world-lab/task/run": "/openenglish.v1.WorldLab/TaskRun",
 "/v1/world-lab/dispatch/request": "/openenglish.v1.WorldLab/DispatchRequest",
 "/v1/world-lab/dispatch/:id/approve": "/openenglish.v1.WorldLab/DispatchApprove",
 "/v1/world-lab/dispatch/:id/deny": "/openenglish.v1.WorldLab/DispatchDeny"
};
  var orig = window.fetch.bind(window);
  window.fetch = function (input, init) {
    try {
      var isReq = typeof Request !== "undefined" && input instanceof Request;
      var raw = isReq ? input.url : String(input);
      var u = new URL(raw, location.href);
      var i = u.pathname.indexOf("/v1/");
      if (i >= 0) {
        var base = u.pathname.slice(0, i);
        var path = u.pathname.slice(i);
        var method = ((init && init.method) || (isReq ? input.method : "GET")).toUpperCase();
        var same = u.origin === location.origin;
        var map = method === "GET" || method === "HEAD" ? GET : POST;
        var target = map[path];
        if (target && (same || path.indexOf("/v1/public/") === 0 || path === "/v1/custom-qa")) {
          var headers = new Headers((init && init.headers) || (isReq ? input.headers : undefined));
          var body = init && init.body !== undefined ? init.body : undefined;
          if (method === "GET" || method === "HEAD") {
            body = "{}";
          }
          headers.set("content-type", same ? "application/json" : "text/plain");
          var next = Object.assign({}, init || {}, { method: "POST", headers: headers, body: body });
          return orig(u.origin + base + target + u.search, next);
        }
      }
    } catch (e) { /* 書き換えに失敗したら素通し */ }
    return orig(input, init);
  };
})();
