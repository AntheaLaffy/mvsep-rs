var B=Object.defineProperty;var M=(t,e,s)=>e in t?B(t,e,{enumerable:!0,configurable:!0,writable:!0,value:s}):t[e]=s;var r=(t,e,s)=>M(t,typeof e!="symbol"?e+"":e,s);(function(){const e=document.createElement("link").relList;if(e&&e.supports&&e.supports("modulepreload"))return;for(const o of document.querySelectorAll('link[rel="modulepreload"]'))i(o);new MutationObserver(o=>{for(const n of o)if(n.type==="childList")for(const a of n.addedNodes)a.tagName==="LINK"&&a.rel==="modulepreload"&&i(a)}).observe(document,{childList:!0,subtree:!0});function s(o){const n={};return o.integrity&&(n.integrity=o.integrity),o.referrerPolicy&&(n.referrerPolicy=o.referrerPolicy),o.crossOrigin==="use-credentials"?n.credentials="include":o.crossOrigin==="anonymous"?n.credentials="omit":n.credentials="same-origin",n}function i(o){if(o.ep)return;o.ep=!0;const n=s(o);fetch(o.href,n)}})();function q(t,e=!1){return window.__TAURI_INTERNALS__.transformCallback(t,e)}async function p(t,e={},s){return window.__TAURI_INTERNALS__.invoke(t,e,s)}var L;(function(t){t.WINDOW_RESIZED="tauri://resize",t.WINDOW_MOVED="tauri://move",t.WINDOW_CLOSE_REQUESTED="tauri://close-requested",t.WINDOW_DESTROYED="tauri://destroyed",t.WINDOW_FOCUS="tauri://focus",t.WINDOW_BLUR="tauri://blur",t.WINDOW_SCALE_FACTOR_CHANGED="tauri://scale-change",t.WINDOW_THEME_CHANGED="tauri://theme-changed",t.WINDOW_CREATED="tauri://window-created",t.WEBVIEW_CREATED="tauri://webview-created",t.DRAG_ENTER="tauri://drag-enter",t.DRAG_OVER="tauri://drag-over",t.DRAG_DROP="tauri://drag-drop",t.DRAG_LEAVE="tauri://drag-leave"})(L||(L={}));async function U(t,e){window.__TAURI_EVENT_PLUGIN_INTERNALS__.unregisterListener(t,e),await p("plugin:event|unlisten",{event:t,eventId:e})}async function A(t,e,s){var i;const o=(i=void 0)!==null&&i!==void 0?i:{kind:"Any"};return p("plugin:event|listen",{event:t,target:o,handler:q(e)}).then(n=>async()=>U(t,n))}async function I(t={}){return typeof t=="object"&&Object.freeze(t),await p("plugin:dialog|open",{options:t})}async function j(t,e){await p("plugin:opener|open_url",{url:t,with:e})}async function V(t,e){await p("plugin:opener|open_path",{path:t,with:e})}async function W(t){return p("plugin:opener|reveal_item_in_dir",{paths:typeof t=="string"?[t]:t})}const $={"fresh-cyan":{id:"fresh-cyan",name:"Fresh Cyan",colors:{primary:"#0891B2",primaryLight:"#22D3EE",primaryLighter:"#A5F3FC",secondary:"#0E7490",cta:"#06B6D4",bgPrimary:"#ECFEFF",bgSecondary:"#CFFAFE",bgTertiary:"#A5F3FC",textPrimary:"#164E63",textSecondary:"#155E75",textMuted:"#0E7490",border:"#A5F3FC"},fonts:{heading:"Nunito",body:"Poppins"}},"kawaii-pink":{id:"kawaii-pink",name:"Kawaii Pink",colors:{primary:"#F472B6",primaryLight:"#F9A8D4",primaryLighter:"#FCE7F3",secondary:"#DB2777",cta:"#EC4899",bgPrimary:"#FDF2F8",bgSecondary:"#FCE7F3",bgTertiary:"#FBCFE8",textPrimary:"#831843",textSecondary:"#9D174D",textMuted:"#BE185D",border:"#FBCFE8"},fonts:{heading:"Nunito",body:"Poppins"}},"elegant-purple":{id:"elegant-purple",name:"Elegant Purple",colors:{primary:"#7C3AED",primaryLight:"#A78BFA",primaryLighter:"#EDE9FE",secondary:"#6D28D9",cta:"#8B5CF6",bgPrimary:"#FAF5FF",bgSecondary:"#F3E8FF",bgTertiary:"#EDE9FE",textPrimary:"#5B21B6",textSecondary:"#6D28D9",textMuted:"#7C3AED",border:"#DDD6FE"},fonts:{heading:"Nunito",body:"Poppins"}},dark:{id:"dark",name:"Dark Mode",colors:{primary:"#22D3EE",primaryLight:"#67E8F9",primaryLighter:"#164E63",secondary:"#0891B2",cta:"#06B6D4",bgPrimary:"#0F172A",bgSecondary:"#1E293B",bgTertiary:"#334155",textPrimary:"#F1F5F9",textSecondary:"#CBD5E1",textMuted:"#94A3B8",border:"#334155"},fonts:{heading:"Nunito",body:"Poppins"}}};function P(t){const e=document.documentElement,s=t.colors;Object.entries(s).forEach(([i,o])=>{const n=`--color-${i.replace(/([A-Z])/g,"-$1").toLowerCase()}`;e.style.setProperty(n,o)}),e.style.setProperty("--font-heading",t.fonts.heading),e.style.setProperty("--font-body",t.fonts.body),e.setAttribute("data-theme",t.id),localStorage.setItem("theme",t.id)}function z(){const t=localStorage.getItem("theme");return t&&$[t]?(P($[t]),$[t]):(P($["fresh-cyan"]),$["fresh-cyan"])}function _(){return Object.values($)}const K={name:"MVSEP",title:"Music Separation Tool"},Q={home:"Home",tasks:"Tasks",algorithms:"Algorithms",settings:"Settings",about:"About",appearance:"Appearance",logs:"Logs"},G={dropzone:"Drop audio files here",selectFile:"or click to select files",initialLoadingHint:"Algorithms, formats, and queue info are loading in the background. You can already start using the UI.",presetLoadLabel:"Preset",algorithm:"Algorithm",option1:"Option 1",option2:"Option 2",option3:"Option 3",format:"Format",runHint:"Create Task uploads and starts polling only; One-click Run creates task, waits for completion, then downloads automatically.",uploading:"Uploading",uploadFailed:"Upload failed. Please retry.",createPending:"Creating...",createTask:"Create Task",createTaskSucceeded:"Task created and background polling started.",createTaskFailed:"Create task failed. Check settings or logs.",runPending:"Starting...",oneClickRun:"One-click Run",runSucceeded:"One-click run completed and auto download finished.",runFailed:"One-click run failed. Please check logs.",start:"Start Separation",publishDemo:"Publish to demo page"},J={current:"Current Tasks",history:"History",centerHint:'Task Center includes all tasks created from Home via "Create Task" and "One-click Run". In-progress tasks are polled automatically and archived to history after completion.',emptyInProgress:"No in-progress tasks",emptyCompleted:"No completed tasks",queueCountLabel:"In queue",queueOrderLabel:"Position",downloadTarget:"Download target",downloadAll:"All files",fileLabel:"File",newestFirst:"Newest",oldestFirst:"Oldest",clearHistory:"Clear History",noHistory:"No history yet",openFolder:"Open Folder",status:{queueing:"Queueing",separating:"Separating",downloading:"Downloading",waiting:"Waiting",distributing:"Distributing",processing:"Processing",merging:"Merging",done:"Done",failed:"Failed"},action:{view:"View Details",cancel:"Cancel",download:"Download",delete:"Delete",retry:"Retry"}},Z={title:"Settings",api:"API Settings",token:"API Token",showToken:"Show API Key",hideToken:"Hide API Key",apiKeyLink:"Get API Token",apiKeyHelpAria:"Show API token guide",apiKeyHelpText:"If your browser has no MVSEP login session, you may be redirected to the homepage. Please log in first, click your username at the top-right, then choose API from the dropdown; or return to GUI and click the link again.",apiKeyHelpShown:"API token guide shown.",testConnection:"Test Connection",connectionSucceeded:"Connection test succeeded.",connectionFailed:"Connection test failed. Check token, API URL, and proxy.",connected:"Connected",disconnected:"Disconnected",server:"Server",mirror:"Mirror",proxy:"Proxy Settings",proxyMode:"Proxy Mode",system:"System Proxy",manual:"Manual",none:"No Proxy",host:"Host",port:"Port",output:"Output Settings",outputDir:"Output Directory",outputFormat:"Output Format",interval:"Poll Interval",algorithmAutoRefreshDays:"Algorithm Auto Refresh",daysUnit:"days",testing:"Testing...",notTested:"Not tested",getApiKey:"Get API Key",apiKeyHelpTitle:"How to get API Key",apiKeyHelp:"The first jump may be blocked by the main site. Sign in to MVSEP first, click your account name at the top-right, then choose API in the dropdown. Or return to this GUI and click the link again.",chooseFolder:"Choose Folder",choosingFolder:"Choosing output folder...",outputDirUpdated:"Output directory updated.",chooseFolderFailed:"Failed to choose output directory.",save:"Save Settings",saving:"Saving settings...",saved:"Settings saved.",saveFailed:"Failed to save settings.",cache:"Cache",algorithmCachePath:"Algorithm Cache File Path"},Y={title:"Appearance",theme:"Theme",selectTheme:"Select Theme",customTheme:"Custom Theme"},X={version:"Version",builtWith:"Built with",docs:"Documentation",info:"Information"},ee={search:"Search Algorithms",searchPlaceholder:"Search algorithms...",group:"Algorithm Groups",models:"Model Options",searchResults:"Search Results",loadingParams:"Loading algorithm parameters...",refreshParams:"Refresh List",refreshList:"Refresh List",refreshListRunning:"Refreshing algorithm list from local cache...",refreshListFailed:"Failed to refresh list from local cache.",fetchLatestInfo:"Fetch Latest Algorithm Info",fetchLatestRunning:"Fetching latest algorithm info from remote...",fetchLatestFailed:"Failed to fetch latest algorithm info. Check network, token, or proxy.",cacheMissing:"Local algorithm cache is missing. Please click Fetch Latest Algorithm Info first.",cacheRefreshed:"Algorithm list refreshed from local cache.",latestFetched:"Latest algorithm info fetched and written to local cache.",noAddOpt:"Current algorithm has no add_opt parameters",presetTitle:"Preset Management",presetNamePlaceholder:"Preset name, e.g. Karaoke-HighQuality",saveCurrentConfig:"Save Current Config",noPresets:"No presets yet. Configure parameters on Home and save one.",noneFound:"No algorithms found. Configure token/API first.",noModelParams:"This algorithm has no selectable model parameters",loadingGuardNotice:"Loading algorithm parameters. Please avoid switching language for a few seconds.",switchLocaleBlockedNotice:"Language switching is temporarily blocked while algorithm parameters are loading."},te={all:"All",inProgress:"In Progress",completed:"Completed",seconds:"seconds",cancel:"Cancel",save:"Save",confirm:"Confirm",close:"Close",collapse:"Collapse",load:"Load",apply:"Apply",use:"Use",loading:"Loading...",error:"Error",success:"Success"},se={title:"Logs",frontend:"Frontend",backend:"Backend",search:"Search logs",all:"All",clear:"Clear",export:"Export",copy:"Copy",debugMode:"Debug Mode"},ie={app:K,nav:Q,home:G,task:J,settings:Z,appearance:Y,about:X,algorithm:ee,common:te,logs:se},oe={name:"MVSEP",title:"音乐分离工具"},ne={home:"首页",tasks:"任务",algorithms:"算法",settings:"设置",about:"关于",appearance:"外观",logs:"日志"},ae={dropzone:"拖拽音频文件到这里",selectFile:"或点击选择文件",initialLoadingHint:"正在后台加载算法/格式/队列信息，界面已可先操作。",presetLoadLabel:"预设加载",algorithm:"算法",option1:"选项1",option2:"选项2",option3:"选项3",format:"格式",runHint:"创建任务：仅上传并加入任务中心轮询；一键运行：创建后自动等待并下载。",uploading:"上传中",uploadFailed:"上传失败，请重试。",createPending:"创建中...",createTask:"创建任务",createTaskSucceeded:"任务已创建，已开始后台轮询。",createTaskFailed:"创建任务失败，请检查设置或查看日志。",runPending:"启动中...",oneClickRun:"一键运行",runSucceeded:"一键运行完成，自动下载已执行。",runFailed:"一键运行失败，请查看日志定位问题。",start:"开始分离",publishDemo:"发布到演示页面"},re={current:"当前任务",history:"历史任务",centerHint:"任务中心包含首页“创建任务”和“一键运行”产生的全部任务；进行中任务会自动轮询状态，完成后归档到历史。",emptyInProgress:"暂无进行中的任务",emptyCompleted:"暂无已完成任务",queueCountLabel:"队列中",queueOrderLabel:"当前排位",downloadTarget:"下载目标",downloadAll:"全部文件",fileLabel:"文件",newestFirst:"最新优先",oldestFirst:"最早优先",clearHistory:"清空历史",noHistory:"暂无历史记录",openFolder:"打开目录",status:{queueing:"排队中",separating:"分离中",downloading:"下载中",waiting:"等待中",distributing:"分发中",processing:"处理中",merging:"合并中",done:"已完成",failed:"失败"},action:{view:"查看详情",cancel:"取消任务",download:"下载",delete:"删除",retry:"重新处理"}},le={title:"设置",api:"API 设置",token:"API Token",showToken:"显示 API Key",hideToken:"隐藏 API Key",apiKeyLink:"获取 API Token",apiKeyHelpAria:"查看 API 获取说明",apiKeyHelpText:"若浏览器没有 MVSEP 登录记录，会被拦截到主站。请先登录后点击右上角用户名，在下拉菜单选择 API；或者回到 GUI 再次点击该链接。",apiKeyHelpShown:"已显示 API 获取说明。",testConnection:"测试连接",connectionSucceeded:"连接测试成功。",connectionFailed:"连接测试失败，请检查 Token、API 地址和代理。",connected:"已连接",disconnected:"未连接",server:"服务器",mirror:"镜像",proxy:"代理设置",proxyMode:"代理模式",system:"系统代理",manual:"手动代理",none:"无代理",host:"主机",port:"端口",output:"输出设置",outputDir:"输出目录",outputFormat:"输出格式",interval:"轮询间隔",algorithmAutoRefreshDays:"算法自动刷新周期",daysUnit:"天",testing:"连接中...",notTested:"未测试",getApiKey:"获取 API Key",apiKeyHelpTitle:"获取 API Key 指引",apiKeyHelp:"首次跳转可能被主站拦截。请先登录 MVSEP 账号，点击右上角账号名称，在下拉菜单进入 API 页面；或者回到 GUI 再次点击“获取 API Key”链接。",chooseFolder:"选择文件夹",choosingFolder:"正在选择输出目录...",outputDirUpdated:"输出目录已更新。",chooseFolderFailed:"选择输出目录失败。",save:"保存设置",saving:"正在保存设置...",saved:"设置已保存。",saveFailed:"保存设置失败。",cache:"缓存",algorithmCachePath:"算法缓存文件路径"},de={title:"外观",theme:"主题设置",selectTheme:"选择主题",customTheme:"自定义主题"},ce={version:"版本",builtWith:"构建",docs:"使用文档",info:"信息"},ue={search:"搜索算法",searchPlaceholder:"输入算法名...",group:"算法组",models:"模型选项",searchResults:"搜索结果",loadingParams:"正在加载算法参数...",refreshParams:"刷新列表",refreshList:"刷新列表",refreshListRunning:"正在从本地缓存刷新算法列表...",refreshListFailed:"刷新列表失败，请检查本地缓存。",fetchLatestInfo:"获取最新算法信息",fetchLatestRunning:"正在从远端拉取最新算法信息...",fetchLatestFailed:"获取最新算法信息失败，请检查网络、Token 或代理。",cacheMissing:"本地算法缓存不存在，请先点击“获取最新算法信息”。",cacheRefreshed:"已从本地缓存刷新算法列表。",latestFetched:"已获取最新算法信息并写入本地缓存。",noAddOpt:"当前算法无 add_opt 参数",presetTitle:"常用预设管理",presetNamePlaceholder:"预设名称，例如：Karaoke-高质量",saveCurrentConfig:"保存当前配置",noPresets:"暂无预设。先在首页选择参数后保存。",noneFound:"未找到算法，请先配置 Token/API。",noModelParams:"该算法没有可选模型参数",loadingGuardNotice:"正在加载算法参数，请在几秒内不要切换语言。",switchLocaleBlockedNotice:"算法参数加载中，暂时禁止切换语言。"},he={all:"全部",inProgress:"进行中",completed:"已完成",seconds:"秒",cancel:"取消",save:"保存",confirm:"确认",close:"关闭",collapse:"收起",load:"加载",apply:"应用",use:"使用",loading:"加载中...",error:"错误",success:"成功"},me={title:"日志",frontend:"前端",backend:"后端",search:"搜索日志",all:"全部",clear:"清空",export:"导出",copy:"复制",debugMode:"调试模式"},ge={app:oe,nav:ne,home:ae,task:re,settings:le,appearance:de,about:ce,algorithm:ue,common:he,logs:me},pe={name:"MVSEP",title:"音楽分離ツール"},fe={home:"ホーム",tasks:"タスク",algorithms:"アルゴリズム",settings:"設定",about:"について",appearance:"外観",logs:"ログ"},ye={dropzone:"オーディオファイルをここにドロップ",selectFile:"またはクリックしてファイルを選択",initialLoadingHint:"アルゴリズム/フォーマット/キュー情報をバックグラウンドで読み込み中です。先に操作を開始できます。",presetLoadLabel:"プリセット読み込み",algorithm:"アルゴリズム",option1:"オプション1",option2:"オプション2",option3:"オプション3",format:"フォーマット",runHint:"タスク作成はアップロードしてポーリング開始のみ、ワンクリック実行は完了待ち後に自動ダウンロードします。",uploading:"アップロード中",uploadFailed:"アップロードに失敗しました。再試行してください。",createPending:"作成中...",createTask:"タスク作成",createTaskSucceeded:"タスクを作成し、バックグラウンド監視を開始しました。",createTaskFailed:"タスク作成に失敗しました。設定またはログを確認してください。",runPending:"開始中...",oneClickRun:"ワンクリック実行",runSucceeded:"ワンクリック実行が完了し、自動ダウンロードも完了しました。",runFailed:"ワンクリック実行に失敗しました。ログを確認してください。",start:"分離開始",publishDemo:"デモページに公開"},be={current:"現在のタスク",history:"履歴",centerHint:"タスクセンターには、ホームの「タスク作成」と「ワンクリック実行」で作成された全タスクが表示されます。進行中タスクは自動ポーリングされ、完了後に履歴へアーカイブされます。",emptyInProgress:"進行中のタスクはありません",emptyCompleted:"完了済みタスクはありません",queueCountLabel:"キュー内",queueOrderLabel:"現在順位",downloadTarget:"ダウンロード対象",downloadAll:"すべてのファイル",fileLabel:"ファイル",newestFirst:"新しい順",oldestFirst:"古い順",clearHistory:"履歴をクリア",noHistory:"履歴はまだありません",openFolder:"フォルダーを開く",status:{queueing:"待機列",separating:"分離中",downloading:"ダウンロード中",waiting:"待機中",distributing:"配信中",processing:"処理中",merging:"マージ中",done:"完了",failed:"失敗"},action:{view:"詳細を見る",cancel:"キャンセル",download:"ダウンロード",delete:"削除",retry:"再試行"}},ve={title:"設定",api:"API設定",token:"APIトークン",testConnection:"接続テスト",connected:"接続済み",disconnected:"未接続",server:"サーバー",mirror:"ミラー",proxy:"プロキシ設定",proxyMode:"プロキシモード",system:"システムプロキシ",manual:"手動",none:"プロキシなし",host:"ホスト",port:"ポート",output:"出力設定",outputDir:"出力ディレクトリ",outputFormat:"出力フォーマット",interval:"ポーリング間隔",algorithmAutoRefreshDays:"アルゴリズム自動更新周期",daysUnit:"日",testing:"接続中...",notTested:"未テスト",showToken:"表示",hideToken:"非表示",apiKeyLink:"API Token を取得",apiKeyHelpAria:"API 取得手順を表示",apiKeyHelpText:"ブラウザに MVSEP のログインセッションがない場合、トップページにリダイレクトされることがあります。先にログインし、右上のユーザー名からドロップダウンの API を選択してください。もしくは GUI に戻ってこのリンクを再度クリックしてください。",apiKeyHelpShown:"API 取得手順を表示しました。",chooseFolder:"フォルダー選択",choosingFolder:"出力フォルダを選択中...",outputDirUpdated:"出力フォルダを更新しました。",chooseFolderFailed:"出力フォルダの選択に失敗しました。",save:"設定を保存",saving:"設定を保存中...",saved:"設定を保存しました。",saveFailed:"設定の保存に失敗しました。",connectionSucceeded:"接続テストに成功しました。",connectionFailed:"接続テストに失敗しました。Token、API URL、プロキシを確認してください。",cache:"キャッシュ",algorithmCachePath:"アルゴリズムキャッシュファイルパス"},ke={title:"外観",theme:"テーマ設定",selectTheme:"テーマを選択",customTheme:"カスタムテーマ"},xe={version:"バージョン",builtWith:"ビルド",docs:"ドキュメント",info:"情報"},we={search:"アルゴリズム検索",searchPlaceholder:"アルゴリズム名を入力...",group:"アルゴリズムグループ",models:"モデルオプション",searchResults:"検索結果",loadingParams:"アルゴリズムパラメータを読み込み中...",refreshParams:"リストを更新",refreshList:"リストを更新",refreshListRunning:"ローカルキャッシュからアルゴリズム一覧を更新中...",refreshListFailed:"ローカルキャッシュからの一覧更新に失敗しました。",fetchLatestInfo:"最新アルゴリズム情報を取得",fetchLatestRunning:"リモートから最新アルゴリズム情報を取得中...",fetchLatestFailed:"最新アルゴリズム情報の取得に失敗しました。ネットワーク、Token、プロキシを確認してください。",cacheMissing:"ローカルアルゴリズムキャッシュがありません。先に「最新アルゴリズム情報を取得」を実行してください。",cacheRefreshed:"ローカルキャッシュからアルゴリズム一覧を更新しました。",latestFetched:"最新アルゴリズム情報を取得し、ローカルキャッシュに保存しました。",noAddOpt:"このアルゴリズムには add_opt パラメータがありません",presetTitle:"プリセット管理",presetNamePlaceholder:"プリセット名（例: Karaoke-HighQuality）",saveCurrentConfig:"現在設定を保存",noPresets:"プリセットはまだありません。ホームで設定して保存してください。",noneFound:"アルゴリズムが見つかりません。まず Token/API を設定してください。",noModelParams:"このアルゴリズムには選択可能なモデルパラメータがありません",loadingGuardNotice:"アルゴリズムパラメータを読み込み中です。数秒間は言語切替を行わないでください。",switchLocaleBlockedNotice:"アルゴリズムパラメータ読み込み中のため、言語切替は一時的に無効です。"},$e={all:"すべて",inProgress:"進行中",completed:"完了",seconds:"秒",cancel:"キャンセル",save:"保存",confirm:"確認",close:"閉じる",collapse:"折りたたむ",load:"読み込む",apply:"適用",use:"使用",loading:"読み込み中...",error:"エラー",success:"成功"},Te={title:"ログ",frontend:"フロントエンド",backend:"バックエンド",search:"ログ検索",all:"すべて",clear:"クリア",export:"エクスポート",copy:"コピー",debugMode:"デバッグモード"},Fe={app:pe,nav:fe,home:ye,task:be,settings:ve,appearance:ke,about:xe,algorithm:we,common:$e,logs:Te},D={en:ie,"zh-CN":ge,ja:Fe},Pe={en:{code:"en",name:"English",nativeName:"EN"},"zh-CN":{code:"zh-CN",name:"Chinese",nativeName:"中文"},ja:{code:"ja",name:"Japanese",nativeName:"日本語"}};let x="en";function C(t){x=t,localStorage.setItem("locale",t),document.documentElement.lang=t}function H(){return x}function Se(){const t=localStorage.getItem("locale");if(t&&D[t])return x=t,document.documentElement.lang=t,t;const e=navigator.language;return e.startsWith("zh")?x="zh-CN":e.startsWith("ja")?x="ja":x="en",document.documentElement.lang=x,x}function l(t){const e=t.split(".");let s=D[x];for(const i of e)if(s&&typeof s=="object"&&i in s)s=s[i];else return t;return typeof s=="string"?s:t}function Le(){return Object.values(Pe)}function T(t,e){try{const s=localStorage.getItem(t);return s?JSON.parse(s):e}catch{return e}}function F(t,e){try{return localStorage.setItem(t,JSON.stringify(e)),!0}catch{return!1}}function Ae(t){try{return localStorage.getItem(t)}catch{return null}}function Ie(t,e){try{return localStorage.setItem(t,e),!0}catch{return!1}}function O(t,e,s){let i=t;if(e!=="all"&&(i=i.filter(o=>o.level===e)),s){const o=s.toLowerCase();i=i.filter(n=>n.message.toLowerCase().includes(o)||n.timestamp.toLowerCase().includes(o))}return i}function He(t){const e=t.currentLogType==="frontend"?t.frontendLogs:t.backendLogs,s=O(e,t.logFilterLevel,t.logSearchQuery),i=s.map(o=>{const n=o.level==="ERROR"?"text-red-500":o.level==="WARN"?"text-yellow-500":o.level==="DEBUG"?"text-gray-500":"text-blue-500";return`<div class="py-1 border-b border-border last:border-0">
        <span class="text-text-muted">[${o.timestamp}]</span>
        <span class="${n} ml-2">[${o.level}]</span>
        <span class="text-text-primary ml-2">${t.escapeHtml(o.message)}</span>
      </div>`}).join("");return`
      <div class="space-y-4">
        <div class="card">
          <div class="flex items-center justify-between mb-4 flex-wrap gap-2">
            <div class="flex gap-2">
              <button class="btn ${t.currentLogType==="frontend"?"btn-primary":"btn-secondary"}" data-action="switch-log-type" data-type="frontend">
                ${t.t("logs.frontend")}
              </button>
              <button class="btn ${t.currentLogType==="backend"?"btn-primary":"btn-secondary"}" data-action="switch-log-type" data-type="backend">
                ${t.t("logs.backend")}
              </button>
            </div>
            <div class="flex gap-2 items-center">
              <input type="text" class="input w-48" id="log-search" placeholder="${t.t("logs.search")||"Search..."}" value="${t.escapeHtml(t.logSearchQuery)}">
              <select class="select w-28" id="log-level-filter">
                <option value="all" ${t.logFilterLevel==="all"?"selected":""}>${t.t("logs.all")||"All"}</option>
                <option value="INFO" ${t.logFilterLevel==="INFO"?"selected":""}>INFO</option>
                <option value="WARN" ${t.logFilterLevel==="WARN"?"selected":""}>WARN</option>
                <option value="ERROR" ${t.logFilterLevel==="ERROR"?"selected":""}>ERROR</option>
                <option value="DEBUG" ${t.logFilterLevel==="DEBUG"?"selected":""}>DEBUG</option>
              </select>
              <button class="btn btn-secondary" data-action="clear-logs">🗑️ ${t.t("logs.clear")}</button>
              <button class="btn btn-secondary" data-action="export-logs">📥 ${t.t("logs.export")}</button>
              <button class="btn btn-secondary" data-action="copy-logs">📋 ${t.t("logs.copy")}</button>
            </div>
          </div>
          <div class="bg-bg-primary rounded-lg p-4 h-[500px] overflow-auto font-mono text-sm" id="log-container">
            ${i||'<p class="text-text-muted">No logs available</p>'}
          </div>
          <div class="mt-2 text-sm text-text-muted">
            Showing ${s.length} of ${e.length} entries
          </div>
        </div>
      </div>
    `}function _e(t){return`
      <div class="space-y-6">
        <div class="card text-center py-8">
          <div class="text-6xl mb-4">🎵</div>
          <h2 class="text-3xl font-bold text-primary">MVSEP</h2>
          <p class="text-text-secondary mb-4">${t("app.title")}</p>
          <p class="text-text-muted">Version: 0.1.7</p>
          <p class="text-text-muted">Author: 如月风铃</p>
          <p class="text-text-muted">Protocol: APLv2</p>
        </div>
        
        <div class="card">
          <h3 class="font-semibold text-text-primary mb-4">📖 ${t("about.docs")}</h3>
          <ul class="space-y-2 text-text-secondary">
            <li><button class="text-primary underline" data-action="open-url" data-url="https://github.com/AntheaLaffy/mvsep-rs">GitHub: github.com/AntheaLaffy/mvsep-rs</button></li>
            <li><button class="text-primary underline" data-action="open-url" data-url="https://mvsep.com">API: mvsep.com</button></li>
          </ul>
        </div>
        
        <div class="card">
          <h3 class="font-semibold text-text-primary mb-4">ℹ️ ${t("about.info")}</h3>
          <p class="text-text-secondary">
            MVSEP CLI - Music vocal/instrumental separation tool.<br/>
            Uses MVSEP API for audio separation.<br/>
            Supports multiple algorithms and model options.
          </p>
        </div>
        
        <div class="text-center text-text-muted">APLv2</div>
      </div>
    `}function De(t){return`
      <div class="flex items-center justify-between mb-4 flex-wrap gap-2">
        <div class="flex gap-2">
          <button class="btn ${t.taskViewFilter==="inProgress"?"btn-primary":"btn-secondary"}" data-action="set-task-filter" data-filter="inProgress">
            ${t.t("common.inProgress")} (${t.inProgressCount})
          </button>
          <button class="btn ${t.taskViewFilter==="completed"?"btn-primary":"btn-secondary"}" data-action="set-task-filter" data-filter="completed">
            ${t.t("common.completed")} (${t.completedCount})
          </button>
          <button class="btn ${t.taskViewFilter==="history"?"btn-primary":"btn-secondary"}" data-action="set-task-filter" data-filter="history">
            ${t.t("task.history")} (${t.historyCount})
          </button>
        </div>
        <div class="flex gap-2 items-center ${t.taskViewFilter==="history"?"":"hidden"}">
          <input type="text" class="input w-48" id="history-search" placeholder="${t.t("algorithm.searchPlaceholder")||"Search..."}" value="${t.historySearchQueryEscaped}">
          <select class="select w-28" id="history-sort">
            <option value="desc" ${t.historySortOrder==="desc"?"selected":""}>${t.t("task.newestFirst")||"Newest"}</option>
            <option value="asc" ${t.historySortOrder==="asc"?"selected":""}>${t.t("task.oldestFirst")||"Oldest"}</option>
          </select>
          <button class="btn btn-secondary" data-action="clear-history">🗑️ ${t.t("task.clearHistory")||"Clear"}</button>
        </div>
      </div>
    `}function Ce(t){return t.taskViewFilter==="inProgress"?`
        <div class="card">
          <h3 class="font-semibold text-text-primary mb-4">${t.t("common.inProgress")}</h3>
          ${t.inProgressTasks.length===0?`
            <p class="text-text-muted text-center py-4">${t.t("task.emptyInProgress")}</p>
          `:`
            <div class="space-y-3" id="tasks-current-task-list">
              ${t.inProgressTasks.map(e=>t.renderTaskCard(e)).join("")}
            </div>
          `}
        </div>
      `:t.taskViewFilter==="completed"?`
        <div class="card">
          <h3 class="font-semibold text-text-primary mb-4">${t.t("common.completed")}</h3>
          ${t.completedTasks.length===0?`
            <p class="text-text-muted text-center py-4">${t.t("task.emptyCompleted")}</p>
          `:`
            <div class="space-y-3" id="tasks-current-task-list">
              ${t.completedTasks.map(e=>t.renderTaskCard(e)).join("")}
            </div>
          `}
        </div>
      `:`
      <div class="card">
        <h3 class="font-semibold text-text-primary mb-4">${t.t("task.history")}</h3>
        ${t.historyRecords.length===0?`
          <p class="text-text-muted text-center py-4">${t.t("task.noHistory")||"No history yet"}</p>
        `:`
          <div class="space-y-3">
            ${t.historyRecords.map(e=>t.renderHistoryCard(e)).join("")}
          </div>
          ${t.totalPages>1?`
            <div class="flex items-center justify-center gap-2 mt-4">
              <button class="btn btn-secondary" data-action="history-prev" ${t.historyPage===0?"disabled":""}>
                ← Previous
              </button>
              <span class="text-text-muted">${t.historyPage+1} / ${t.totalPages}</span>
              <button class="btn btn-secondary" data-action="history-next" ${t.historyPage>=t.totalPages-1?"disabled":""}>
                Next →
              </button>
            </div>
          `:""}
        `}
      </div>
    `}function Oe(t){return`
      <div class="space-y-4">
        <div class="card">
          <p class="text-sm text-text-muted mb-3">
            ${t.t("task.centerHint")}
          </p>
          <div id="tasks-filter-bar">${t.filterBarHtml}</div>
        </div>
        <div id="tasks-view-content">
          ${t.viewContentHtml}
        </div>
      </div>
    `}function Ee(t){return t.currentTasks.length===0?"":`
      <div class="card">
        <h3 class="font-semibold text-text-primary mb-4">${t.t("task.current")}</h3>
        <div class="space-y-3" id="home-current-task-list">
          ${t.currentTasks.map(e=>t.renderTaskCard(e)).join("")}
        </div>
      </div>
    `}function Re(t){return t.recentHistory.length===0?"":`
      <div class="card">
        <h3 class="font-semibold text-text-primary mb-4">${t.t("task.history")}</h3>
        <div class="space-y-3">
          ${t.recentHistory.map(e=>t.renderHistoryCard(e)).join("")}
        </div>
      </div>
    `}function Ne(t){const{task:e}=t;return`
      <div class="p-4 bg-bg-primary rounded-lg border border-border" data-task-hash="${e.hash}">
        <div class="flex items-center justify-between mb-2">
          <span class="font-medium">🎧 ${t.escapeHtml(e.file_name)}</span>
          <span class="status-badge ${t.statusClass}">${t.statusText}</span>
        </div>
        ${t.phase==="separating"?`
          <div class="progress-bar mb-2">
            <div class="progress-bar-fill" style="width: ${e.progress}%"></div>
          </div>
          <p class="text-sm text-text-muted">${e.progress.toFixed(0)}%</p>
        `:""}
        ${t.phase==="queueing"?`
          <p class="text-sm text-text-muted mb-2">
            ${t.t("task.queueCountLabel")}: ${e.queue_count??t.queueQueuedFallback??"-"}，${t.t("task.queueOrderLabel")}: ${e.current_order??"-"}
          </p>
        `:""}
        ${t.phase==="downloading"?`
          <div class="progress-bar mb-2">
            <div class="progress-bar-fill" style="width: ${e.download_percent??0}%"></div>
          </div>
          <p class="text-sm text-text-muted mb-1">
            ${e.download_file_name?`${t.t("task.fileLabel")}: ${t.escapeHtml(e.download_file_name)} / `:""}${(e.download_percent??0).toFixed(1)}%
          </p>
          <p class="text-sm text-text-muted">
            ${t.formatBytes(e.download_bytes)} / ${t.formatBytes(e.download_total_bytes)} · ${t.formatBytes(e.download_speed_bps)}/s
          </p>
        `:""}
        ${e.status==="done"?`
          <p class="text-sm text-text-muted mb-2">${e.output_files.length} files</p>
          ${e.output_files.length>0?`
            <div class="mb-2">
              <label class="block text-xs text-text-muted mb-1">${t.t("task.downloadTarget")}</label>
              <select class="select text-sm" data-action="select-download-target" data-hash="${e.hash}">
                <option value="-1" ${t.selectedDownloadIndex===null?"selected":""}>${t.t("task.downloadAll")}</option>
                ${e.output_files.map((s,i)=>`
                  <option value="${i}" ${t.selectedDownloadIndex===i?"selected":""}>${i+1}. ${t.escapeHtml(t.getDisplayFileName(s))}</option>
                `).join("")}
              </select>
            </div>
          `:""}
        `:""}
        ${e.status==="failed"?`
          <p class="text-sm text-red-500 mb-2">${t.escapeHtml(e.error||"Failed")}</p>
        `:""}
        ${e.message?`<p class="text-sm text-text-muted mb-2">${t.escapeHtml(e.message)}</p>`:""}
        ${t.isExpanded?`
          <div class="mt-2 p-3 rounded-lg bg-bg-secondary border border-border text-sm text-text-secondary">
            <p>Hash: ${t.escapeHtml(e.hash)}</p>
            <p>Algorithm: ${t.escapeHtml(e.algorithm_name)} (#${e.algorithm_id})</p>
            <p>Model: ${t.escapeHtml(e.model_name||"-")}</p>
            <p>Format: ${t.escapeHtml(t.formatName)}</p>
            <p>Created: ${new Date(e.created_at).toLocaleString()}</p>
          </div>
        `:""}
        <div class="flex gap-2 mt-2">
          <button class="btn btn-secondary text-sm" data-action="toggle-task-details">
            ${t.isExpanded?t.t("common.close"):t.t("task.action.view")}
          </button>
          ${e.status==="done"?`
            <button class="btn btn-primary text-sm" data-action="download-task">
              📥 ${t.t("task.action.download")}
            </button>
          `:""}
          ${t.phase==="downloading"?`
            <button class="btn btn-secondary text-sm" data-action="cancel-download" ${t.isCancellingDownload?"disabled":""}>
              ${t.isCancellingDownload?`${t.t("common.loading")}...`:t.t("task.action.cancel")}
            </button>
          `:""}
          ${(e.status==="done"||e.status==="failed")&&t.phase!=="downloading"?`
            <button class="btn btn-secondary text-sm" data-action="delete-task">
              🗑️ ${t.t("task.action.delete")}
            </button>
          `:""}
        </div>
      </div>
    `}function Be(t){const{record:e}=t;return`
      <div class="p-4 bg-bg-primary rounded-lg border border-border">
        <div class="flex items-center justify-between mb-2">
          <span class="font-medium">🎧 ${t.escapeHtml(e.fileName)}</span>
          <span class="status-badge ${t.statusClass}">${t.statusText}</span>
        </div>
        <div class="text-sm text-text-muted space-y-1">
          <p>📊 ${t.escapeHtml(e.algorithmName)} ${e.modelName?`(${t.escapeHtml(e.modelName)})`:""}</p>
          <p>📁 ${t.escapeHtml(e.formatName)}</p>
          <p>🕐 ${t.completedDate}</p>
          ${e.outputFiles.length>0?`<p>📄 ${e.outputFiles.length} files</p>`:""}
          ${e.error?`<p class="text-red-500">❌ ${t.escapeHtml(e.error)}</p>`:""}
        </div>
        <div class="flex gap-2 mt-3">
          ${e.status==="done"?`
            <button class="btn btn-secondary text-sm" data-action="open-history-output" data-id="${e.id}">
              📂 ${t.t("task.openFolder")||"Open"}
            </button>
          `:""}
          <button class="btn btn-secondary text-sm" data-action="retry-task" data-id="${e.id}">
            🔄 ${t.t("task.action.retry")}
          </button>
        </div>
      </div>
    `}function Me(t){const{config:e,formats:s,connectionStatus:i,connectionStatusText:o,algorithmCachePath:n,isTokenVisible:a,isTestingConnection:u,isChoosingOutputDir:h,isSavingSettings:f,t:c}=t;return`
      <div class="space-y-6 max-w-2xl">
        <div class="card">
          <h3 class="font-semibold text-text-primary mb-4">🔑 ${c("settings.api")}</h3>
          <div class="space-y-4">
            <div>
              <label class="block text-sm text-text-secondary mb-1">${c("settings.token")}</label>
              <div class="flex items-center gap-2">
                <input type="${a?"text":"password"}" class="input" id="token-input" value="${(e==null?void 0:e.token)||""}" />
                <button
                  class="btn btn-secondary whitespace-nowrap"
                  type="button"
                  data-action="toggle-token-visibility"
                  aria-label="${c(a?"settings.hideToken":"settings.showToken")}"
                >
                  ${c(a?"settings.hideToken":"settings.showToken")}
                </button>
              </div>
              <div class="mt-2 flex items-center gap-2 text-sm">
                <button class="text-primary underline" type="button" data-action="open-url" data-url="https://mvsep.com/user-api">
                  ${c("settings.apiKeyLink")}
                </button>
                <button
                  class="btn btn-secondary px-2 py-1 text-xs"
                  type="button"
                  data-action="show-api-help"
                  aria-label="${c("settings.apiKeyHelpAria")}"
                >
                  ?
                </button>
              </div>
            </div>
            <div>
              <label class="block text-sm text-text-secondary mb-1">${c("settings.server")}</label>
              <input type="text" class="input" id="api-url-input" value="${(e==null?void 0:e.api_url)||"https://mvsep.com"}" />
            </div>
            <div>
              <label class="block text-sm text-text-secondary mb-1">${c("settings.mirror")}</label>
              <select class="select" id="mirror-select">
                <option value="main" ${(e==null?void 0:e.mirror)==="main"?"selected":""}>Main (mvsep.com)</option>
                <option value="mirror" ${(e==null?void 0:e.mirror)==="mirror"?"selected":""}>Mirror</option>
              </select>
            </div>
            <div class="flex items-center gap-2">
              <button class="btn btn-secondary" data-action="test-connection" ${u||i==="testing"?"disabled":""}>
                ${c(u||i==="testing"?"settings.testing":"settings.testConnection")}
              </button>
              <span class="text-sm px-2 py-1 rounded border ${i==="success"?"bg-green-100 text-green-700 border-green-200":i==="error"?"bg-red-100 text-red-700 border-red-200":i==="testing"?"bg-blue-100 text-blue-700 border-blue-200":"bg-bg-primary text-text-muted border-border"}">
                ${o||c("settings.notTested")}
              </span>
            </div>
          </div>
        </div>

        <div class="card">
          <h3 class="font-semibold text-text-primary mb-4">📁 ${c("settings.output")}</h3>
          <div class="space-y-4">
            <div>
              <label class="block text-sm text-text-secondary mb-1">${c("settings.outputDir")}</label>
              <div class="flex gap-2">
                <input type="text" class="input" id="output-dir-input" value="${(e==null?void 0:e.output_dir)||"./output"}" />
                <button class="btn btn-secondary whitespace-nowrap" data-action="choose-output-dir" ${h?"disabled":""}>
                  ${c(h?"common.loading":"settings.chooseFolder")}
                </button>
              </div>
            </div>
            <div>
              <label class="block text-sm text-text-secondary mb-1">${c("settings.interval")} (${c("common.seconds")})</label>
              <input type="number" class="input" id="poll-interval-input" value="${(e==null?void 0:e.poll_interval)||5}" />
            </div>
            <div>
              <label class="block text-sm text-text-secondary mb-1">${c("settings.algorithmAutoRefreshDays")} (${c("settings.daysUnit")})</label>
              <input
                type="number"
                min="1"
                class="input"
                id="algo-auto-refresh-days-input"
                value="${(e==null?void 0:e.algorithm_auto_refresh_days)||15}"
              />
            </div>
            <div>
              <label class="block text-sm text-text-secondary mb-1">${c("settings.outputFormat")}</label>
              <select class="select" id="settings-output-format-select">
                ${s.map(m=>`
                  <option value="${m.id}" ${m.id===((e==null?void 0:e.output_format)??1)?"selected":""}>${m.name}</option>
                `).join("")}
              </select>
            </div>
          </div>
        </div>

        <div class="card">
          <h3 class="font-semibold text-text-primary mb-4">🌐 ${c("settings.proxy")}</h3>
          <div class="space-y-4">
            <div>
              <label class="block text-sm text-text-secondary mb-1">${c("settings.proxyMode")}</label>
              <select class="select" id="proxy-mode-select">
                <option value="system" ${(e==null?void 0:e.proxy_mode)==="system"?"selected":""}>${c("settings.system")}</option>
                <option value="manual" ${(e==null?void 0:e.proxy_mode)==="manual"?"selected":""}>${c("settings.manual")}</option>
                <option value="none" ${(e==null?void 0:e.proxy_mode)==="none"?"selected":""}>${c("settings.none")}</option>
              </select>
            </div>
            <div class="grid grid-cols-2 gap-4">
              <div>
                <label class="block text-sm text-text-secondary mb-1">${c("settings.host")}</label>
                <input type="text" class="input" id="proxy-host-input" value="${(e==null?void 0:e.proxy_host)||"127.0.0.1"}" />
              </div>
              <div>
                <label class="block text-sm text-text-secondary mb-1">${c("settings.port")}</label>
                <input type="text" class="input" id="proxy-port-input" value="${(e==null?void 0:e.proxy_port)||"7897"}" />
              </div>
            </div>
          </div>
        </div>

        <div class="card">
          <h3 class="font-semibold text-text-primary mb-4">🗂️ ${c("settings.cache")}</h3>
          <div>
            <label class="block text-sm text-text-secondary mb-1">${c("settings.algorithmCachePath")}</label>
            <input
              type="text"
              class="input"
              readonly
              value="${n||"-"}"
            />
          </div>
        </div>

        <button class="btn btn-primary w-full" data-action="save-settings" ${f?"disabled":""}>
          ${c(f?"common.loading":"settings.save")}
        </button>
      </div>
    `}function qe(t){return`
      <div class="space-y-6">
        <div class="card">
          <h3 class="font-semibold text-text-primary mb-4">🎨 ${t.t("appearance.selectTheme")}</h3>
          <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
            ${t.themes.map(e=>`
              <div class="p-4 rounded-lg border-2 cursor-pointer transition-all hover:shadow-custom ${e.id===t.currentThemeId?"border-primary":"border-border"}"
                   data-action="select-theme" data-theme-id="${e.id}">
                <div class="h-16 rounded-lg mb-2" style="background: ${e.colors.bgPrimary}; border: 1px solid ${e.colors.border}">
                  <div class="h-full flex items-center justify-center gap-1">
                    <div class="w-4 h-4 rounded-full" style="background: ${e.colors.primary}"></div>
                    <div class="w-4 h-4 rounded-full" style="background: ${e.colors.primaryLight}"></div>
                    <div class="w-4 h-4 rounded-full" style="background: ${e.colors.secondary}"></div>
                  </div>
                </div>
                <p class="text-sm text-center text-text-primary">${e.name}</p>
              </div>
            `).join("")}
            <div class="p-4 rounded-lg border-2 ${t.currentThemeId==="custom"?"border-primary":"border-border"}">
              <div class="h-16 rounded-lg mb-2 border border-border" style="background: ${t.draft.bgPrimary}">
                <div class="h-full flex items-center justify-center gap-1">
                  <div class="w-4 h-4 rounded-full" style="background: ${t.draft.primary}"></div>
                  <div class="w-4 h-4 rounded-full" style="background: ${t.draft.textPrimary}"></div>
                </div>
              </div>
              <p class="text-sm text-center text-text-primary">${t.t("appearance.customTheme")}</p>
            </div>
          </div>
        </div>

        <div class="card">
          <h3 class="font-semibold text-text-primary mb-4">🔧 ${t.t("appearance.customTheme")}</h3>
          <div class="grid md:grid-cols-3 gap-4">
            <label class="text-sm text-text-secondary">
              Primary
              <input id="custom-theme-primary" type="color" class="input h-11 p-1 mt-1" value="${t.draft.primary}">
            </label>
            <label class="text-sm text-text-secondary">
              Background
              <input id="custom-theme-bg" type="color" class="input h-11 p-1 mt-1" value="${t.draft.bgPrimary}">
            </label>
            <label class="text-sm text-text-secondary">
              Text
              <input id="custom-theme-text" type="color" class="input h-11 p-1 mt-1" value="${t.draft.textPrimary}">
            </label>
          </div>
          <div class="flex gap-2 mt-4">
            <button class="btn btn-primary" data-action="save-custom-theme">${t.t("common.save")}</button>
            <button class="btn btn-secondary" data-action="reset-custom-theme">Reset</button>
          </div>
        </div>
      </div>
    `}function Ue(t){return`
      <div class="space-y-6">
        ${t.isInitialLoading?`
          <div class="card">
            <p class="text-sm text-text-secondary">${t.t("home.initialLoadingHint")}</p>
          </div>
        `:""}
        <div class="card">
          <div id="dropzone" class="dropzone" data-action="select-file">
            <div class="text-4xl mb-4">🎶</div>
            <p class="text-lg text-text-primary mb-2">${t.t("home.dropzone")}</p>
            <p class="text-text-muted">${t.t("home.selectFile")}</p>
            ${t.selectedFile?`
              <div class="mt-4 px-4 py-2 bg-bg-tertiary rounded-lg">
                📁 ${t.selectedFileLabelEscaped}
              </div>
            `:""}
            <button class="btn btn-primary mt-4" data-action="select-file">
              ${t.t("home.selectFile")}
            </button>
          </div>
        </div>

        ${t.selectedFile?`
          <div class="card space-y-4">
            <h3 class="font-semibold text-text-primary">${t.t("home.algorithm")}</h3>
            <select class="select" id="algorithm-select">
              ${t.algorithmSelectOptionsHtml}
            </select>

            ${t.algorithmOptionsHtml}

            ${t.presetBlockHtml}

            <h3 class="font-semibold text-text-primary">${t.t("home.format")}</h3>
            <select class="select" id="format-select">
              ${t.formatOptionsHtml}
            </select>

            <label class="flex items-center gap-2">
              <input type="checkbox" id="demo-checkbox" ${t.isDemo?"checked":""}>
              <span>${t.t("home.publishDemo")}</span>
            </label>

            <label class="block text-sm text-text-secondary">
              Run Timeout (s)
              <input type="number" class="input mt-1" id="run-timeout-input" min="30" value="${t.runTimeoutSeconds}">
            </label>

            <p class="text-xs text-text-muted">
              ${t.t("home.runHint")}
            </p>
            ${t.isSubmittingTask?t.uploadInfoHtml:""}

            <button class="btn btn-cta w-full" data-action="start-separation" ${t.isSubmittingTask?"disabled":""}>
              ${t.submittingAction==="create"?t.t("home.createPending"):t.t("home.createTask")}
            </button>
            <button class="btn btn-primary w-full" data-action="start-run-workflow" ${t.isSubmittingTask?"disabled":""}>
              ${t.submittingAction==="run"?t.t("home.runPending"):t.t("home.oneClickRun")}
            </button>
          </div>
        `:""}
        <div id="home-current-tasks-section">${t.currentTasksSectionHtml}</div>
        <div id="home-recent-history-section">${t.recentHistorySectionHtml}</div>
      </div>
    `}function je(t){const{details:e,escapeHtml:s,t:i}=t;return e?e.fields.length===0?`<p class="text-sm text-text-muted mt-3">${i("algorithm.noModelParams")}</p>`:`
      <div class="mt-3 p-3 rounded-lg bg-bg-secondary border border-border">
        ${e.fields.map(o=>`
          <div class="mb-2 last:mb-0">
            <p class="text-sm font-semibold text-text-primary">${o.text||o.name} (--${o.name})</p>
            <p class="text-xs text-text-muted mt-1">
              ${Object.entries(o.options).map(([n,a])=>`${n}: ${s(a)}`).join(" | ")}
            </p>
          </div>
        `).join("")}
      </div>
    `:`<p class="text-sm text-text-muted mt-3">${i("common.loading")}</p>`}function Ve(t){return`
      <div class="space-y-4">
        <div class="card">
          <div class="flex gap-2 items-center">
            <input
              type="text"
              class="input"
              id="algorithm-search-input"
              placeholder="${t.t("algorithm.searchPlaceholder")}"
              value="${t.algorithmSearchQueryEscaped}"
            />
            <button class="btn btn-secondary whitespace-nowrap" data-action="fetch-latest-algo-info" ${t.isFetchingLatest?"disabled":""}>
              ${t.isFetchingLatest?t.t("common.loading"):t.t("algorithm.fetchLatestInfo")}
            </button>
            <button class="btn btn-secondary whitespace-nowrap" data-action="refresh-algo-list" ${t.isRefreshingList?"disabled":""}>
              ${t.isRefreshingList?t.t("common.loading"):t.t("algorithm.refreshList")}
            </button>
          </div>
        </div>

        <div class="card space-y-3">
          <h3 class="font-semibold text-text-primary">${t.t("algorithm.presetTitle")}</h3>
          <div class="flex gap-2">
            <input class="input" id="preset-name-input" placeholder="${t.t("algorithm.presetNamePlaceholder")}" value="${t.presetNameInputEscaped}" />
            <button class="btn btn-primary whitespace-nowrap" data-action="save-preset">${t.t("algorithm.saveCurrentConfig")}</button>
          </div>
          ${t.presets.length>0?`
            <div class="space-y-2">
              ${t.presets.map(e=>`
                <div class="p-3 rounded-lg bg-bg-primary border border-border flex items-center justify-between gap-2">
                  <div class="text-sm">
                    <p class="font-medium">${t.escapeHtml(e.name)}</p>
                    <p class="text-text-muted">${t.t("home.algorithm")} #${e.algorithmId} / opt1=${e.opt1??"-"} opt2=${e.opt2??"-"} opt3=${e.opt3??"-"}</p>
                  </div>
                  <div class="flex gap-2">
                    <button class="btn btn-secondary text-sm" data-action="apply-algo-preset" data-id="${e.id}">${t.t("common.apply")}</button>
                    <button class="btn btn-secondary text-sm" data-action="delete-preset" data-id="${e.id}">${t.t("task.action.delete")}</button>
                  </div>
                </div>
              `).join("")}
            </div>
          `:`<p class="text-sm text-text-muted">${t.t("algorithm.noPresets")}</p>`}
        </div>
        
        <div class="space-y-4">
          ${t.groups.length===0?`
            <div class="card text-text-muted">${t.t("algorithm.noneFound")}</div>
          `:t.groups.map(e=>`
            <div class="card">
              <h3 class="font-semibold text-text-primary mb-3">${e.name}</h3>
              <div class="space-y-2">
                ${e.algorithms.map(s=>`
                  <div class="p-3 bg-bg-primary rounded-lg border border-border">
                    <div class="flex items-center justify-between gap-2">
                      <span class="text-text-primary">${s.id}: ${s.name}</span>
                      <div class="flex gap-2">
                        <button class="btn btn-secondary text-sm" data-action="toggle-algo-details" data-id="${s.id}">
                          ${t.expandedAlgorithmId===s.id?t.t("common.collapse"):t.t("task.action.view")}
                        </button>
                        <button class="btn btn-primary text-sm" data-action="use-algorithm" data-id="${s.id}">
                          ${t.t("common.use")}
                        </button>
                      </div>
                    </div>
                    ${t.expandedAlgorithmId===s.id?t.renderAlgorithmDetailsSection(s.id):""}
                  </div>
                `).join("")}
              </div>
            </div>
          `).join("")}
        </div>
      </div>
    `}function We(t,e){var i,o,n,a,u,h,f,c,m,k,w,y,b;const s=t.getTargetElement(e);if(s){if(s.closest("[data-nav]")){const g=s.closest("[data-nav]").dataset.nav;g&&t.navigate(g)}if(s.closest('[data-action="select-file"]')){t.selectFile();return}if(s.closest('[data-action="start-separation"]')&&t.startSeparation(),s.closest('[data-action="start-run-workflow"]')&&t.runSeparationWorkflow(),s.closest('[data-action="select-theme"]')){const d=s.closest("[data-theme-id]"),g=d==null?void 0:d.dataset.themeId;if(g){const v=_().find(N=>N.id===g);v&&(P(v),t.initializeCustomThemeDraft(),t.render())}}if(s.closest('[data-action="select-locale"]')){const d=s.closest("[data-locale]"),g=d==null?void 0:d.dataset.locale;g&&(C(g),t.render())}if(s.closest('[data-action="download-task"]')){const d=s.closest("[data-task-hash]"),g=d==null?void 0:d.dataset.taskHash;if(g){const v=t.selectedDownloadFileIndexByHash.get(g)??null;t.downloadTask(g,v)}}if(s.closest('[data-action="cancel-download"]')){const d=s.closest("[data-task-hash]"),g=d==null?void 0:d.dataset.taskHash;g&&t.cancelDownload(g)}if(s.closest('[data-action="delete-task"]')){const d=s.closest("[data-task-hash]"),g=d==null?void 0:d.dataset.taskHash;g&&t.deleteTask(g)}if(s.closest('[data-action="toggle-task-details"]')){const d=s.closest("[data-task-hash]"),g=d==null?void 0:d.dataset.taskHash;g&&(t.expandedTaskHashes.has(g)?t.expandedTaskHashes.delete(g):t.expandedTaskHashes.add(g),t.render())}if(s.closest('[data-action="test-connection"]')&&t.testConnection(),s.closest('[data-action="choose-output-dir"]')&&t.chooseOutputDir(),s.closest('[data-action="save-settings"]')&&t.saveSettings(),s.closest('[data-action="toggle-token-visibility"]')&&t.toggleTokenVisibility(),s.closest('[data-action="show-api-help"]')&&t.showApiTokenHelp(),s.closest('[data-action="switch-log-type"]')){const d=(i=s.closest('[data-action="switch-log-type"]'))==null?void 0:i.getAttribute("data-type");d&&t.switchLogType(d)}if(s.closest('[data-action="clear-logs"]')&&t.clearLogs(),s.closest('[data-action="export-logs"]')&&t.exportLogs(),s.closest('[data-action="copy-logs"]')&&t.copyLogs(),s.closest('[data-action="clear-history"]')){if(!t.confirmAction("Clear all task history records? This cannot be undone."))return;t.clearHistory(),t.render()}if(s.closest('[data-action="set-task-filter"]')){const d=(o=s.closest('[data-action="set-task-filter"]'))==null?void 0:o.getAttribute("data-filter");(d==="inProgress"||d==="completed"||d==="history")&&(t.taskViewFilter=d,t.render())}if(s.closest('[data-action="delete-history"]')){const d=(n=s.closest('[data-action="delete-history"]'))==null?void 0:n.getAttribute("data-id");if(d){if(!t.confirmAction("Delete this history record? This cannot be undone."))return;t.deleteFromHistory(d),t.render()}}if(s.closest('[data-action="retry-task"]')){const d=(a=s.closest('[data-action="retry-task"]'))==null?void 0:a.getAttribute("data-id");d&&t.retryFromHistory(d)}if(s.closest('[data-action="open-output"]')){const d=(u=s.closest('[data-action="open-output"]'))==null?void 0:u.getAttribute("data-path");d&&t.openOutput(d)}if(s.closest('[data-action="open-history-output"]')){const d=(h=s.closest('[data-action="open-history-output"]'))==null?void 0:h.getAttribute("data-id");d&&t.openHistoryOutput(d)}if(s.closest('[data-action="toggle-algo-details"]')){const d=(f=s.closest('[data-action="toggle-algo-details"]'))==null?void 0:f.getAttribute("data-id");if(d){const g=parseInt(d,10),v=t.expandedAlgorithmId!==g;t.expandedAlgorithmId=v?g:null,v?t.loadAlgorithmDetails(g).then(()=>t.render()):t.render()}}if(s.closest('[data-action="use-algorithm"]')){const d=(c=s.closest('[data-action="use-algorithm"]'))==null?void 0:c.getAttribute("data-id");d&&(t.selectedAlgorithm=parseInt(d,10),t.selectedOpt1=null,t.selectedOpt2=null,t.selectedOpt3=null,t.navigate("home"),t.loadAlgorithmDetails(t.selectedAlgorithm).then(()=>t.render()))}if(s.closest('[data-action="refresh-algo-list"]')&&t.refreshAlgorithmListFromLocal(),s.closest('[data-action="fetch-latest-algo-info"]')&&t.fetchLatestAlgorithmInfo(),s.closest('[data-action="save-preset"]')){const d=document.getElementById("preset-name-input"),g=((m=d==null?void 0:d.value)==null?void 0:m.trim())||t.presetNameInput.trim();if(!g)return;const v=t.createPresetFromCurrent(g);t.presets.unshift(v),t.selectedHomePresetId=v.id,t.presetNameInput="",t.savePresets(),t.render()}if(s.closest('[data-action="apply-home-preset"]')&&t.selectedHomePresetId&&t.applyPresetById(t.selectedHomePresetId),s.closest('[data-action="apply-algo-preset"]')){const d=(k=s.closest('[data-action="apply-algo-preset"]'))==null?void 0:k.getAttribute("data-id");d&&t.applyPresetById(d)}if(s.closest('[data-action="delete-preset"]')){const d=(w=s.closest('[data-action="delete-preset"]'))==null?void 0:w.getAttribute("data-id");if(d){if(!t.confirmAction("Delete this preset? This cannot be undone."))return;t.presets=t.presets.filter(g=>g.id!==d),t.selectedHomePresetId===d&&(t.selectedHomePresetId=((y=t.presets[0])==null?void 0:y.id)||""),t.savePresets(),t.render()}}if(s.closest('[data-action="open-url"]')){const d=(b=s.closest('[data-action="open-url"]'))==null?void 0:b.getAttribute("data-url");d&&j(d)}if(s.closest('[data-action="open-logs-page"]')&&(t.navigate("logs"),t.switchLogType("backend"),t.render()),s.closest('[data-action="save-custom-theme"]')&&t.saveCustomTheme(),s.closest('[data-action="reset-custom-theme"]')&&(t.initializeCustomThemeDraft(),t.render()),s.closest('[data-action="history-prev"]')&&t.historyPage>0&&(t.historyPage--,t.render()),s.closest('[data-action="history-next"]')){const d=Math.ceil(t.getFilteredHistory().length/t.historyPageSize);t.historyPage<d-1&&(t.historyPage++,t.render())}}}function ze(t,e){const s=t.getTargetElement(e);s&&(s.id==="log-search"&&(t.logSearchQuery=s.value,t.render()),s.id==="algorithm-search-input"&&t.onAlgorithmSearchInput(s.value),s.id==="history-search"&&(t.historySearchQuery=s.value,t.historyPage=0,t.render()),(s.id==="custom-theme-primary"||s.id==="custom-theme-bg"||s.id==="custom-theme-text")&&(t.customThemeDraft||t.initializeCustomThemeDraft(),t.customThemeDraft&&(s.id==="custom-theme-primary"&&(t.customThemeDraft.primary=s.value),s.id==="custom-theme-bg"&&(t.customThemeDraft.bgPrimary=s.value),s.id==="custom-theme-text"&&(t.customThemeDraft.textPrimary=s.value))),s.id==="preset-name-input"&&(t.presetNameInput=s.value))}async function Ke(t,e){const s=t.getTargetElement(e);if(s){if(s instanceof HTMLSelectElement&&s.dataset.action==="select-download-target"){const i=s.dataset.hash;if(i){const o=parseInt(s.value,10);t.selectedDownloadFileIndexByHash.set(i,Number.isNaN(o)||o<0?null:o)}return}if(s.id==="log-level-filter"&&(t.logFilterLevel=s.value,t.render()),s.id==="history-sort"&&(t.historySortOrder=s.value,t.historyPage=0,t.render()),s.id==="locale-select"){if(t.isLoadingAlgorithmDetails){t.showTransientNotice(l("algorithm.switchLocaleBlockedNotice"),"warn",2600),t.sendDebugLog("WARN",`locale change blocked while loading algorithm details: requested=${s.value}`);return}t.sendDebugLog("INFO",`locale change requested: ${s.value}`),C(s.value),t.render()}if(s.id==="algorithm-select"){const i=parseInt(s.value,10);t.selectedAlgorithm=i,t.selectedOpt1=null,t.selectedOpt2=null,t.selectedOpt3=null,await t.loadAlgorithmDetails(i),t.render()}if(s.id==="opt1-select"&&(t.selectedOpt1=s.value?parseInt(s.value,10):null),s.id==="opt2-select"&&(t.selectedOpt2=s.value?parseInt(s.value,10):null),s.id==="opt3-select"&&(t.selectedOpt3=s.value?parseInt(s.value,10):null),s.id==="format-select"&&(t.selectedFormat=parseInt(s.value,10)),s.id==="demo-checkbox"&&(t.isDemo=s.checked),s.id==="run-timeout-input"&&(t.runTimeoutSeconds=Math.max(30,parseInt(s.value||"1800",10))),s.id==="settings-output-format-select"&&t.config&&(t.config.output_format=parseInt(s.value,10)),s.id==="mirror-select"&&t.config){t.config.mirror=s.value,t.config.api_url=t.getApiUrlByMirror(s.value);const i=document.getElementById("api-url-input");i&&(i.value=t.config.api_url),t.render()}s.id==="home-preset-select"&&(t.selectedHomePresetId=s.value)}}function Qe(t,e){const s=t.getTargetElement(e);if(!s)return;const i=s.closest("#dropzone");i&&(e.preventDefault(),i.classList.add("active"))}function Ge(t,e){const s=t.getTargetElement(e);if(!s)return;const i=s.closest("#dropzone");i&&i.classList.remove("active")}function Je(t,e){var n;const s=t.getTargetElement(e);if(!s)return;const i=s.closest("#dropzone");if(!i)return;e.preventDefault(),i.classList.remove("active");const o=(n=e.dataTransfer)==null?void 0:n.files;o&&o.length>0&&t.handleFileDrop(o[0])}function Ze(t){if(!t)return!1;const e=String(t).toLowerCase();return e.includes("download cancelled")||e.includes("canceled")||e.includes("cancelled")}function S(t,e){if(!t.taskPollingIntervals.has(e))return;const s=window.setTimeout(async()=>{if(t.taskPollingIntervals.has(e)){if(t.pollInFlightHashes.has(e)){S(t,e);return}await E(t,e),t.taskPollingIntervals.has(e)&&S(t,e)}},t.getPollIntervalSeconds()*1e3);t.taskPollingIntervals.set(e,s)}function Ye(t,e){t.taskPollingIntervals.has(e)||(t.taskPollingIntervals.set(e,-1),S(t,e))}async function E(t,e){var s;if(t.config&&!t.pollInFlightHashes.has(e)){t.pollInFlightHashes.add(e);try{const i=await p("get_task_status",{hash:e,apiUrl:t.config.api_url,token:t.config.token}),o=t.tasks.find(n=>n.hash===e);if(o){const n=t.normalizeTaskStatus(i.status);if(o.status=n,o.progress=Number.isFinite(i.progress)?i.progress:0,o.message=i.message??null,o.queue_count=i.queue_count??null,o.current_order=i.current_order??null,o.phase!=="downloading"){const a=t.getPhaseFromStatus(n);n!=="done"&&n!=="failed"?o.progress>0?o.phase="separating":(o.queue_count??0)>0||(o.current_order??0)>0?o.phase="queueing":o.phase=a:o.phase=a}i.files&&i.files.length>0&&(o.output_files=i.files),i.message&&n==="failed"&&(o.error=i.message)}t.isTerminalStatus((o==null?void 0:o.status)||i.status)&&(R(t,e),o&&t.addToHistory(o,((s=t.config)==null?void 0:s.output_dir)||null)),t.saveActiveTasks(),t.shouldRenderTaskUpdates()&&t.requestTaskPanelsRefresh()}catch(i){console.error("Failed to poll task status:",i)}finally{t.pollInFlightHashes.delete(e)}}}function R(t,e){const s=t.taskPollingIntervals.get(e);s!==void 0&&s>=0&&clearTimeout(s),t.taskPollingIntervals.delete(e),t.pollInFlightHashes.delete(e)}async function Xe(t,e,s=null){if(!t.config)return;t.sendDebugLog("INFO",`downloadTask start: hash=${e}, fileIndex=${s??"all"}`);const i=t.tasks.find(o=>o.hash===e);if(!t.cancellingDownloadHashes.has(e)){i&&(i.phase="downloading",i.download_percent=0,i.download_bytes=0,i.download_total_bytes=null,i.download_speed_bps=0,t.requestTaskPanelsRefresh());try{const o=await p("download_result",{hash:e,outputDir:t.config.output_dir||"./output",fileIndex:s,originalFileName:(i==null?void 0:i.file_name)??null,apiUrl:t.config.api_url,token:t.config.token});if(t.addFrontendLog("INFO",`Downloaded ${o.length} files for task ${e}`),t.sendDebugLog("INFO",`downloadTask success: hash=${e}, files=${o.length}`),i){i.output_files=o,i.status="done",i.phase="done",i.download_percent=100;const n=o[0]||t.config.output_dir||null,a=n?t.getParentPath(n):null;t.addToHistory(i,a)}}catch(o){console.error("Failed to download task:",o),t.sendDebugLog("ERROR",`downloadTask failed: hash=${e}, err=${String(o)}`);const n=String(o).toLowerCase(),a=n.includes("timed out")||n.includes("connection")||n.includes("network")||n.includes("decode")||n.includes("body");i&&(Ze(o)?(i.phase="done",i.download_file_name=null,i.download_speed_bps=0,t.addFrontendLog("INFO",`Download cancelled for task ${e}`)):(i.phase="failed",i.error=a?"Download interrupted. Partial file is kept and can be resumed by clicking Download again.":`Download failed: ${String(o)}`,a&&t.addFrontendLog("WARN",`Download interrupted for task ${e}. Resume is available on next download attempt.`)))}finally{t.saveActiveTasks(),t.requestTaskPanelsRefresh()}}}async function et(t,e){if(!t.cancellingDownloadHashes.has(e)){t.sendDebugLog("INFO",`cancelDownload requested: hash=${e}`),t.cancellingDownloadHashes.add(e),t.requestTaskPanelsRefresh();try{await p("cancel_download",{hash:e}),t.addFrontendLog("INFO",`Cancel requested for download task ${e}`)}catch(s){console.error("Failed to cancel download:",s),t.addFrontendLog("ERROR",`Cancel download failed for task ${e}: ${String(s)}`)}finally{t.cancellingDownloadHashes.delete(e),t.requestTaskPanelsRefresh()}}}class tt{constructor(){r(this,"defaultApiToken","mvsep-demo-key");r(this,"defaultAlgorithmAutoRefreshDays",15);r(this,"currentPage","home");r(this,"config",null);r(this,"tasks",[]);r(this,"algorithms",[]);r(this,"algorithmDetails",new Map);r(this,"formats",[]);r(this,"selectedFile",null);r(this,"selectedAlgorithm",49);r(this,"selectedOpt1",null);r(this,"selectedOpt2",null);r(this,"selectedOpt3",null);r(this,"selectedFormat",1);r(this,"isDemo",!1);r(this,"taskPollingIntervals",new Map);r(this,"pollInFlightHashes",new Set);r(this,"frontendLogs",[]);r(this,"backendLogs",[]);r(this,"currentLogType","frontend");r(this,"logFilterLevel","all");r(this,"logSearchQuery","");r(this,"taskHistory",[]);r(this,"historySearchQuery","");r(this,"historySortOrder","desc");r(this,"historyPage",0);r(this,"historyPageSize",20);r(this,"taskViewFilter","inProgress");r(this,"algorithmSearchQuery","");r(this,"algorithmSearchResults",[]);r(this,"expandedAlgorithmId",null);r(this,"algorithmSearchDebounceTimer",null);r(this,"customThemeDraft",null);r(this,"expandedTaskHashes",new Set);r(this,"queueInfo",null);r(this,"runTimeoutSeconds",1800);r(this,"connectionStatus","idle");r(this,"connectionStatusText","");r(this,"algorithmCachePath",null);r(this,"lastAlgorithmCacheUpdatedAt",null);r(this,"presets",[]);r(this,"selectedHomePresetId","");r(this,"presetNameInput","");r(this,"isSubmittingTask",!1);r(this,"submittingAction",null);r(this,"isInitialLoading",!0);r(this,"uploadFileName","");r(this,"uploadBytes",0);r(this,"uploadTotalBytes",0);r(this,"uploadSpeedBps",0);r(this,"uploadPercent",0);r(this,"uploadFailed",!1);r(this,"isShellMounted",!1);r(this,"lastRenderedPage",null);r(this,"backendLogPollTimer",null);r(this,"queueInfoLastFetchAt",0);r(this,"queueInfoInFlight",null);r(this,"renderSeq",0);r(this,"renderInProgress",!1);r(this,"droppedRenderCount",0);r(this,"renderScheduled",!1);r(this,"renderPending",!1);r(this,"lastSidebarRenderKey",null);r(this,"logStickToBottom",!0);r(this,"lastRenderedLogCountByType",{frontend:0,backend:0});r(this,"isLoadingAlgorithmDetails",!1);r(this,"transientNotice",null);r(this,"transientNoticeTimer",null);r(this,"selectedDownloadFileIndexByHash",new Map);r(this,"cancellingDownloadHashes",new Set);r(this,"runningActions",new Set);r(this,"statusBanner",null);r(this,"statusBannerTimer",null);r(this,"isTokenVisible",!1);r(this,"activeTasksSaveTimer",null);r(this,"activeTasksSaveDebounceMs",500);r(this,"minPollIntervalSeconds",1);r(this,"maxPollIntervalSeconds",60);this.init()}async init(){Se(),z(),this.applySavedCustomThemeIfSelected(),this.initializeCustomThemeDraft(),await this.loadConfig(),this.algorithmSearchResults=this.algorithms,this.setupEventListeners(),this.setupDownloadProgressListener(),this.setupUploadProgressListener(),this.initFrontendLogs(),this.loadTaskHistory(),this.loadActiveTasks(),this.loadPresets(),this.startBackendLogPolling(),this.setupFrontendDebugHooks(),this.render(),this.loadInitialData()}setupDownloadProgressListener(){A("download-progress",e=>{const s=e.payload,i=this.tasks.find(o=>o.hash===s.hash);i&&(i.phase=s.done?"done":"downloading",i.download_file_name=s.file_name,i.download_bytes=s.downloaded_bytes,i.download_total_bytes=s.total_bytes,i.download_speed_bps=s.speed_bps,i.download_percent=s.percent,this.shouldRenderTaskUpdates()&&this.requestTaskPanelsRefresh())})}setupUploadProgressListener(){A("upload-progress",e=>{const s=e.payload;this.uploadFileName=s.file_name,this.uploadBytes=s.uploaded_bytes,this.uploadTotalBytes=s.total_bytes,this.uploadSpeedBps=s.speed_bps,this.uploadPercent=s.percent,this.uploadFailed=s.failed,s.done&&s.failed&&(this.isSubmittingTask=!1,this.submittingAction=null),this.currentPage==="home"&&this.render()})}setupFrontendDebugHooks(){window.addEventListener("error",e=>{this.sendDebugLog("ERROR",`window.error: ${e.message} @ ${e.filename}:${e.lineno}:${e.colno}`)}),window.addEventListener("unhandledrejection",e=>{this.sendDebugLog("ERROR",`unhandledrejection: ${String(e.reason)}`)})}async sendDebugLog(e,s){try{await p("frontend_debug_log",{level:e,message:s})}catch{}}showTransientNotice(e,s="warn",i=3e3){this.transientNotice={message:e,level:s},this.transientNoticeTimer!==null&&clearTimeout(this.transientNoticeTimer),this.transientNoticeTimer=window.setTimeout(()=>{this.transientNotice=null,this.transientNoticeTimer=null,this.render()},i),this.render()}setStatusBanner(e,s,i={}){const{autoHideMs:o=0,showLogsShortcut:n=e==="error"}=i;this.statusBanner={phase:e,message:s,showLogsShortcut:n},this.statusBannerTimer!==null&&(clearTimeout(this.statusBannerTimer),this.statusBannerTimer=null),o>0&&(this.statusBannerTimer=window.setTimeout(()=>{this.statusBanner=null,this.statusBannerTimer=null,this.render()},o)),this.render()}setActionRunning(e,s){s?this.runningActions.add(e):this.runningActions.delete(e)}isActionRunning(e){return this.runningActions.has(e)}async withUiAction(e,s={},i){if(this.isActionRunning(e))return null;this.setActionRunning(e,!0),s.runningMessage&&this.setStatusBanner("running",s.runningMessage,{autoHideMs:0,showLogsShortcut:!1}),this.render();try{const o=await i();return s.successMessage&&this.setStatusBanner("success",s.successMessage,{autoHideMs:s.autoHideSuccessMs??2200,showLogsShortcut:!1}),o}catch(o){const n=s.errorMessage||this.getErrorMessage(o);throw this.setStatusBanner("error",n,{autoHideMs:0,showLogsShortcut:s.showLogsShortcutOnError!==!1}),o}finally{this.setActionRunning(e,!1),this.render()}}getErrorMessage(e){if(e instanceof Error)return e.message;if(typeof e=="string")return e;if(e&&typeof e=="object"){const s=e.message;if(typeof s=="string")return s;try{return JSON.stringify(e)}catch{return String(e)}}return String(e)}isLocalCacheMissingError(e){const s=e.toLowerCase();return s.includes("os error 2")||s.includes("no such file or directory")||s.includes("没有那个文件或目录")||s.includes("cache")}startBackendLogPolling(){this.backendLogPollTimer!==null&&clearInterval(this.backendLogPollTimer),this.backendLogPollTimer=window.setInterval(()=>{this.currentPage!=="logs"||this.currentLogType!=="backend"||this.loadBackendLogs().then(e=>{e&&this.render()})},1200)}shouldRenderTaskUpdates(){return this.currentPage==="home"||this.currentPage==="tasks"}getInProgressTasks(){return this.tasks.filter(e=>e.status!=="done"&&e.status!=="failed")}getCompletedTasks(){return this.tasks.filter(e=>e.status==="done")}requestTaskPanelsRefresh(){this.shouldRenderTaskUpdates()&&(this.refreshTaskPanels()||this.render())}refreshTaskPanels(){if(this.currentPage==="home"){const e=document.getElementById("home-current-tasks-section"),s=document.getElementById("home-recent-history-section");if(!e||!s)return!1;const i=this.getInProgressTasks(),o=document.getElementById("home-current-task-list");if(!(o&&this.patchTaskCardList(o,i))){const a=this.renderHomeCurrentTasksSection();e.innerHTML!==a&&(e.innerHTML=a)}const n=this.renderHomeRecentHistorySection();return s.innerHTML!==n&&(s.innerHTML=n),!0}if(this.currentPage==="tasks"){const e=document.getElementById("tasks-filter-bar"),s=document.getElementById("tasks-view-content");if(!e||!s)return!1;const i=document.activeElement,o=(i==null?void 0:i.id)||null,n=i instanceof HTMLInputElement||i instanceof HTMLTextAreaElement,a=n?i.selectionStart:null,u=n?i.selectionEnd:null,h=this.renderTasksFilterBar();e.innerHTML!==h&&(e.innerHTML=h);const f=this.taskViewFilter==="inProgress"?this.getInProgressTasks():this.taskViewFilter==="completed"?this.getCompletedTasks():null,c=document.getElementById("tasks-current-task-list");if(!(c&&f&&this.patchTaskCardList(c,f))){const m=this.renderTasksViewContent();s.innerHTML!==m&&(s.innerHTML=m)}if(o&&n){const m=document.getElementById(o);if(m){try{m.focus({preventScroll:!0})}catch{m.focus()}if(a!==null&&u!==null)try{m.setSelectionRange(a,u)}catch{}}}return!0}return!1}patchTaskCardList(e,s){const i=Array.from(e.children).filter(o=>o instanceof HTMLElement&&!!o.getAttribute("data-task-hash"));if(i.length!==s.length)return!1;for(let o=0;o<s.length;o++)if((i[o].getAttribute("data-task-hash")||"")!==s[o].hash)return!1;for(let o=0;o<s.length;o++){const n=i[o],a=this.renderTaskCard(s[o]).trim();if(n.outerHTML!==a){const u=document.createElement("div");u.innerHTML=a;const h=u.firstElementChild;if(!h)return!1;n.replaceWith(h)}}return!0}isSameQueueInfo(e,s){return!e&&!s?!0:!e||!s?!1:e.active===s.active&&e.queued===s.queued}renderQueueInfoBadge(){return this.queueInfo?`
      <div class="text-sm px-3 py-1 rounded-full bg-bg-secondary border border-border text-text-secondary">
        Queue: ${this.queueInfo.active} active / ${this.queueInfo.queued} queued
      </div>
    `:""}refreshHeaderQueueInfo(){const e=document.getElementById("queue-info-slot");if(!e)return!1;const s=this.renderQueueInfoBadge();return e.innerHTML!==s&&(e.innerHTML=s),!0}async loadInitialData(){this.isInitialLoading=!0,this.render(),await this.loadAlgorithmCachePath(),await this.refreshAlgorithmListFromLocal({render:!1,syncSearch:!0}).catch(()=>{});const e=[this.loadFormats()];this.shouldLoadQueueInfoForPage(this.currentPage)&&e.push(this.loadQueueInfo(!0)),await Promise.allSettled(e),this.shouldAutoRefreshAlgorithmCache()&&await this.fetchLatestAlgorithmInfo({render:!1,showNotice:!1}),this.isInitialLoading=!1,this.render()}shouldLoadQueueInfoForPage(e){return e==="home"||e==="tasks"}getTargetElement(e){const s=e.target;return s?s instanceof HTMLElement||s instanceof Element?s:s instanceof Node&&s.parentElement?s.parentElement:null:null}getPresetStorageKey(){return"mvsep_presets_v1"}loadPresets(){var e;this.presets=T(this.getPresetStorageKey(),[]),this.selectedHomePresetId=((e=this.presets[0])==null?void 0:e.id)||""}savePresets(){F(this.getPresetStorageKey(),this.presets)||console.error("Failed to save presets")}createPresetFromCurrent(e){return{id:`preset_${Date.now()}`,name:e.trim(),algorithmId:this.selectedAlgorithm,opt1:this.selectedOpt1,opt2:this.selectedOpt2,opt3:this.selectedOpt3,formatId:this.selectedFormat,demo:this.isDemo}}async applyPresetById(e){const s=this.presets.find(i=>i.id===e);s&&(this.selectedAlgorithm=s.algorithmId,this.selectedOpt1=s.opt1,this.selectedOpt2=s.opt2,this.selectedOpt3=s.opt3,this.selectedFormat=s.formatId,this.isDemo=s.demo,this.render())}async loadQueueInfo(e=!1){var n,a;if(!((n=this.config)!=null&&n.token)||!((a=this.config)!=null&&a.api_url))return;const s=this.config.api_url,i=this.config.token,o=Date.now();if(!(!e&&o-this.queueInfoLastFetchAt<4e3))return this.queueInfoInFlight?this.queueInfoInFlight:(this.queueInfoInFlight=(async()=>{const u=this.queueInfo;try{const h=await p("get_queue_info",{apiUrl:s,token:i});this.queueInfo=h,this.queueInfoLastFetchAt=Date.now(),this.isSameQueueInfo(u,this.queueInfo)||(this.refreshHeaderQueueInfo()?this.requestTaskPanelsRefresh():this.render())}catch(h){console.error("Failed to load queue info:",h),this.queueInfo=null,this.isSameQueueInfo(u,this.queueInfo)||(this.refreshHeaderQueueInfo()?this.requestTaskPanelsRefresh():this.render())}finally{this.queueInfoInFlight=null}})(),this.queueInfoInFlight)}getActiveTasksStorageKey(){return"mvsep_active_tasks_v1"}loadActiveTasks(){try{const e=T(this.getActiveTasksStorageKey(),[]);this.tasks=e.filter(s=>s.status!=="done"&&s.status!=="failed");for(const s of this.tasks)s.phase=this.getPhaseFromStatus(s.status);for(const s of this.tasks)this.startPolling(s.hash),this.pollTaskStatus(s.hash)}catch(e){console.error("Failed to load active tasks:",e),this.tasks=[]}}saveActiveTasks(e=!1){if(!e){this.activeTasksSaveTimer!==null&&clearTimeout(this.activeTasksSaveTimer),this.activeTasksSaveTimer=window.setTimeout(()=>{this.activeTasksSaveTimer=null,this.saveActiveTasks(!0)},this.activeTasksSaveDebounceMs);return}this.activeTasksSaveTimer!==null&&(clearTimeout(this.activeTasksSaveTimer),this.activeTasksSaveTimer=null);const s=this.tasks.filter(o=>o.status!=="done"&&o.status!=="failed");F(this.getActiveTasksStorageKey(),s)||console.error("Failed to save active tasks")}loadTaskHistory(){this.taskHistory=T("mvsep_task_history",[])}saveTaskHistory(){this.taskHistory.length>100&&(this.taskHistory=this.taskHistory.slice(-100)),F("mvsep_task_history",this.taskHistory)||console.error("Failed to save task history")}addToHistory(e,s=null){const i=this.formats.find(a=>a.id===e.format),o={id:e.hash,fileName:e.file_name,algorithmId:e.algorithm_id,algorithmName:e.algorithm_name,modelId:e.model_id,modelName:e.model_name,model2Id:e.model2_id??null,model2Name:e.model2_name??null,model3Id:e.model3_id??null,model3Name:e.model3_name??null,formatId:e.format,formatName:(i==null?void 0:i.name)||"Unknown",status:e.status,createdAt:e.created_at,completedAt:Date.now(),outputFiles:e.output_files,outputPath:s,error:e.error},n=this.taskHistory.findIndex(a=>a.id===o.id);n>=0&&this.taskHistory.splice(n,1),this.taskHistory.unshift(o),this.saveTaskHistory()}deleteFromHistory(e){this.taskHistory=this.taskHistory.filter(s=>s.id!==e),this.saveTaskHistory()}clearHistory(){this.taskHistory=[],this.saveTaskHistory()}initializeCustomThemeDraft(){const e=getComputedStyle(document.documentElement);this.customThemeDraft={primary:e.getPropertyValue("--color-primary").trim()||"#0891B2",bgPrimary:e.getPropertyValue("--color-bg-primary").trim()||"#ECFEFF",textPrimary:e.getPropertyValue("--color-text-primary").trim()||"#164E63"}}applySavedCustomThemeIfSelected(){if(Ae("theme")!=="custom")return;const s=T("mvsep_custom_theme",null);s&&(document.documentElement.style.setProperty("--color-primary",s.primary),document.documentElement.style.setProperty("--color-primary-light",s.primary),document.documentElement.style.setProperty("--color-cta",s.primary),document.documentElement.style.setProperty("--color-bg-primary",s.bgPrimary),document.documentElement.style.setProperty("--color-text-primary",s.textPrimary),document.documentElement.setAttribute("data-theme","custom"),this.customThemeDraft=s)}getFilteredHistory(){let e=[...this.taskHistory];if(this.historySearchQuery){const s=this.historySearchQuery.toLowerCase();e=e.filter(i=>i.fileName.toLowerCase().includes(s)||i.algorithmName.toLowerCase().includes(s))}return e.sort((s,i)=>{const o=s.completedAt||s.createdAt,n=i.completedAt||i.createdAt;return this.historySortOrder==="desc"?n-o:o-n}),e}getPaginatedHistory(){const e=this.getFilteredHistory(),s=this.historyPage*this.historyPageSize;return e.slice(s,s+this.historyPageSize)}async retryFromHistory(e){const s=this.taskHistory.find(i=>i.id===e);s&&(this.selectedAlgorithm=s.algorithmId,this.selectedOpt1=s.modelId,this.selectedOpt2=s.model2Id??null,this.selectedOpt3=s.model3Id??null,this.selectedFormat=s.formatId,this.navigate("home"),await this.loadAlgorithmDetails(s.algorithmId),this.validateSelectedOptionsForCurrentAlgorithm(),this.render(),this.addFrontendLog("INFO",`Retrying task: ${s.fileName}. Refilled algorithm=${s.algorithmName}, opt1=${s.modelName||"-"}, opt2=${s.model2Name||"-"}, opt3=${s.model3Name||"-"}, format=${s.formatName}. Please select source audio on Home page.`))}initFrontendLogs(){this.addFrontendLog("INFO","Application initialized");const e=console.log,s=console.warn,i=console.error;console.log=(...o)=>{e.apply(console,o),this.addFrontendLog("INFO",o.map(n=>String(n)).join(" "))},console.warn=(...o)=>{s.apply(console,o),this.addFrontendLog("WARN",o.map(n=>String(n)).join(" "))},console.error=(...o)=>{i.apply(console,o),this.addFrontendLog("ERROR",o.map(n=>String(n)).join(" "))}}addFrontendLog(e,s){const i={timestamp:new Date().toLocaleString(),level:e,message:s};this.frontendLogs.push(i),this.frontendLogs.length>1e3&&(this.frontendLogs=this.frontendLogs.slice(-1e3))}hasLogListChanged(e,s){if(e.length!==s.length)return!0;for(let i=0;i<e.length;i++){const o=e[i],n=s[i];if(!o||!n||o.timestamp!==n.timestamp||o.level!==n.level||o.message!==n.message)return!0}return!1}async loadBackendLogs(){try{const e=await p("get_backend_logs"),s=this.hasLogListChanged(this.backendLogs,e);return this.backendLogs=e,s}catch(e){return console.error("Failed to load backend logs:",e),!1}}async loadConfig(){try{this.config=await p("load_config"),(!this.config.token||!this.config.token.trim())&&(this.config.token=this.defaultApiToken),this.config.mirror&&(!this.config.api_url||this.config.api_url.includes("mvsep.com"))&&(this.config.api_url=this.getApiUrlByMirror(this.config.mirror)),this.config.algorithm_auto_refresh_days=this.normalizeAlgorithmAutoRefreshDays(this.config.algorithm_auto_refresh_days),this.selectedFormat=this.config.output_format??1}catch(e){console.error("Failed to load config:",e),this.config={token:this.defaultApiToken,api_url:"https://mvsep.com",mirror:"main",proxy_mode:"system",proxy_host:"127.0.0.1",proxy_port:"7897",output_dir:"./output",output_format:1,poll_interval:5,algorithm_auto_refresh_days:this.defaultAlgorithmAutoRefreshDays},this.selectedFormat=1}}async loadAlgorithmCachePath(){try{this.algorithmCachePath=await p("get_algorithm_cache_path_cmd")}catch(e){console.error("Failed to load algorithm cache path:",e),this.algorithmCachePath=null}}async saveConfig(e){try{await p("save_config",{config:e}),this.config=e}catch(s){console.error("Failed to save config:",s)}}filterAlgorithmGroupsFromLocal(e){const s=e.trim().toLowerCase();return s?this.algorithms.map(o=>({...o,algorithms:o.algorithms.filter(n=>n.name.toLowerCase().includes(s))})).filter(o=>o.algorithms.length>0):this.algorithms}getFirstAvailableAlgorithmId(e){for(const s of e)for(const i of s.algorithms)return i.id;return null}hasAlgorithmId(e,s){return e.some(i=>i.algorithms.some(o=>o.id===s))}applyAlgorithmSearchFilter(){this.algorithmSearchResults=this.filterAlgorithmGroupsFromLocal(this.algorithmSearchQuery)}async refreshAlgorithmListFromLocal(e={}){const s=e.render!==!1,i=e.syncSearch!==!1;try{await this.withUiAction("algo-refresh-list",{runningMessage:l("algorithm.refreshListRunning"),successMessage:l("algorithm.cacheRefreshed"),errorMessage:l("algorithm.refreshListFailed")},async()=>{const o=await p("refresh_algorithm_list_from_local");this.lastAlgorithmCacheUpdatedAt=o.updated_at||null,this.algorithms=o.groups,i?this.applyAlgorithmSearchFilter():this.algorithmSearchResults=o.groups;const n=new Set(this.algorithms.flatMap(u=>u.algorithms.map(h=>h.id)));for(const u of this.algorithmDetails.keys())n.has(u)||this.algorithmDetails.delete(u);const a=this.selectedAlgorithm;if(!this.hasAlgorithmId(this.algorithms,this.selectedAlgorithm)){const u=this.getFirstAvailableAlgorithmId(this.algorithms);u!==null&&(this.selectedAlgorithm=u,this.selectedOpt1=null,this.selectedOpt2=null,this.selectedOpt3=null,a!==u&&this.showTransientNotice(`Algorithm ${a} is unavailable. Switched to ${u}.`,"warn",3200))}this.selectedAlgorithm&&!this.algorithmDetails.has(this.selectedAlgorithm)&&await this.loadAlgorithmDetails(this.selectedAlgorithm),this.validateSelectedOptionsForCurrentAlgorithm()})}catch(o){console.error("Failed to refresh algorithm list from local cache:",o);const n=this.getErrorMessage(o);this.isLocalCacheMissingError(n)?this.showTransientNotice(l("algorithm.cacheMissing"),"warn",3400):this.showTransientNotice(n,"warn",3400)}finally{s&&this.render()}}async fetchLatestAlgorithmInfo(e={}){var o,n;if(!((o=this.config)!=null&&o.token)||!((n=this.config)!=null&&n.api_url)){await this.refreshAlgorithmListFromLocal({render:e.render!==!1,syncSearch:!0});return}const s=e.render!==!1,i=e.showNotice!==!1;this.isLoadingAlgorithmDetails=!0,s&&this.render();try{await this.withUiAction("algo-fetch-latest",{runningMessage:l("algorithm.fetchLatestRunning"),successMessage:i?l("algorithm.latestFetched"):"",errorMessage:l("algorithm.fetchLatestFailed")},async()=>{var a,u,h,f,c;await p("fetch_latest_algorithm_info",{apiUrl:(a=this.config)==null?void 0:a.api_url,token:(u=this.config)==null?void 0:u.token,proxyMode:(h=this.config)==null?void 0:h.proxy_mode,proxyHost:(f=this.config)==null?void 0:f.proxy_host,proxyPort:(c=this.config)==null?void 0:c.proxy_port}),await this.refreshAlgorithmListFromLocal({render:!1,syncSearch:!0}),this.selectedAlgorithm&&await this.loadAlgorithmDetails(this.selectedAlgorithm),this.validateSelectedOptionsForCurrentAlgorithm()})}catch(a){console.error("Failed to fetch latest algorithm info:",a),await this.refreshAlgorithmListFromLocal({render:!1,syncSearch:!0});const u=this.getErrorMessage(a);this.showTransientNotice(u,"warn",3200)}finally{this.isLoadingAlgorithmDetails=!1,s&&this.render()}}async loadFormats(){var e,s;if(!((e=this.config)!=null&&e.token)||!((s=this.config)!=null&&s.api_url)){this.formats=[{id:0,name:"MP3 (320 kbps)"},{id:1,name:"WAV (16 bit)"},{id:2,name:"FLAC (16 bit)"},{id:3,name:"M4A (lossy)"},{id:4,name:"WAV (32 bit)"},{id:5,name:"FLAC (24 bit)"}];return}try{this.formats=await p("list_formats",{apiUrl:this.config.api_url,token:this.config.token})}catch(i){console.error("Failed to load formats:",i),this.formats=[{id:0,name:"MP3 (320 kbps)"},{id:1,name:"WAV (16 bit)"},{id:2,name:"FLAC (16 bit)"},{id:3,name:"M4A (lossy)"},{id:4,name:"WAV (32 bit)"},{id:5,name:"FLAC (24 bit)"}]}}async loadAlgorithmDetails(e){if(e<=0)return null;this.sendDebugLog("INFO",`loadAlgorithmDetails start from local cache: id=${e}`);try{const s=await p("get_algorithm_details_from_local",{algorithmId:e});return this.algorithmDetails.set(e,s),this.sendDebugLog("INFO",`loadAlgorithmDetails done: id=${e}, fields=${s.fields.length}`),s}catch(s){console.error("Failed to load algorithm details:",s);const i=this.getErrorMessage(s);return this.isLocalCacheMissingError(i)?this.showTransientNotice(l("algorithm.cacheMissing"),"warn",3600):this.showTransientNotice(i,"warn",3200),this.sendDebugLog("ERROR",`loadAlgorithmDetails failed: id=${e}, err=${i}`),null}}onAlgorithmSearchInput(e){this.algorithmSearchQuery=e,this.algorithmSearchDebounceTimer&&(clearTimeout(this.algorithmSearchDebounceTimer),this.algorithmSearchDebounceTimer=null),this.algorithmSearchDebounceTimer=window.setTimeout(()=>{this.applyAlgorithmSearchFilter(),this.render()},300)}setupEventListeners(){const e=this;document.addEventListener("click",s=>We(e,s)),document.addEventListener("input",s=>ze(e,s)),document.addEventListener("change",s=>{Ke(e,s)}),document.addEventListener("dragover",s=>Qe(e,s)),document.addEventListener("dragleave",s=>Ge(e,s)),document.addEventListener("drop",s=>Je(e,s))}navigate(e){this.currentPage=e,this.shouldLoadQueueInfoForPage(e)&&this.loadQueueInfo(!1),this.render()}async selectFile(){try{const e=await I({multiple:!1,filters:[{name:"Audio",extensions:["wav","mp3","flac","ogg","m4a","aac"]}]});e&&(this.selectedFile=e,this.render())}catch(e){console.error("Failed to select file:",e)}}handleFileDrop(e){const s=e.path;this.selectedFile=s||e.name,this.render()}async startSeparation(){if(this.isSubmittingTask)return;this.sendDebugLog("INFO","startSeparation requested"),this.setStatusBanner("running",l("home.createPending"),{showLogsShortcut:!1});const e=await this.createTaskFromCurrentSelection("create");e&&(this.startPolling(e),this.addFrontendLog("INFO",`Task created: ${e}. Background polling started.`),this.sendDebugLog("INFO",`startSeparation created task: ${e}`),this.setStatusBanner("success",l("home.createTaskSucceeded"),{autoHideMs:2200,showLogsShortcut:!1}),this.render())}async runSeparationWorkflow(){if(this.isSubmittingTask)return;this.sendDebugLog("INFO","runSeparationWorkflow requested"),this.setStatusBanner("running",l("home.runPending"),{showLogsShortcut:!1});const e=await this.createTaskFromCurrentSelection("run");if(e){this.addFrontendLog("INFO",`One-click run started: ${e}`),this.sendDebugLog("INFO",`runSeparationWorkflow created task: ${e}`);try{const s=await this.waitForTaskCompletion(e,this.runTimeoutSeconds);this.isSuccessStatus(s.status)?(this.addFrontendLog("INFO",`Task completed, starting auto download: ${e}`),this.sendDebugLog("INFO",`runSeparationWorkflow task completed, auto download start: ${e}`),await this.downloadTask(e),this.setStatusBanner("success",l("home.runSucceeded"),{autoHideMs:2400,showLogsShortcut:!1})):(this.addFrontendLog("WARN",`One-click run ended with non-success status: ${s.status}`),this.sendDebugLog("WARN",`runSeparationWorkflow non-success status: ${s.status}`),this.setStatusBanner("error",l("home.runFailed"),{showLogsShortcut:!0}))}catch(s){console.error("Run workflow failed:",s),this.addFrontendLog("ERROR",`One-click run failed: ${String(s)}`),this.sendDebugLog("ERROR",`runSeparationWorkflow failed: ${String(s)}`),this.setStatusBanner("error",l("home.runFailed"),{showLogsShortcut:!0})}finally{this.render()}}}async createTaskFromCurrentSelection(e){if(this.isSubmittingTask||!this.selectedFile||!this.config)return null;if(!this.selectedFile.includes("/")&&!this.selectedFile.includes("\\"))return alert("Drag-and-drop did not expose file path. Please use file picker."),null;const s=this.algorithmDetails.get(this.selectedAlgorithm);if((s==null?void 0:s.fields.some(o=>o.name==="add_opt1"))&&this.selectedOpt1===null)return alert("Please select an option for --opt1"),null;this.isSubmittingTask=!0,this.submittingAction=e,this.uploadFileName=this.selectedFile.split("/").pop()||this.selectedFile.split("\\").pop()||"audio",this.uploadBytes=0,this.uploadTotalBytes=0,this.uploadSpeedBps=0,this.uploadPercent=0,this.uploadFailed=!1,this.addFrontendLog("INFO",`${e==="run"?"Run":"Create"} task started`),this.sendDebugLog("INFO",`createTaskFromCurrentSelection start: action=${e}, algorithm=${this.selectedAlgorithm}`),this.render();try{const o=await p("create_task",{filePath:this.selectedFile,sepType:this.selectedAlgorithm,opt1:this.selectedOpt1,opt2:this.selectedOpt2,opt3:this.selectedOpt3,outputFormat:this.selectedFormat,demo:this.isDemo,apiUrl:this.config.api_url,token:this.config.token}),n={hash:o,file_name:this.selectedFile.split("/").pop()||"unknown",algorithm_id:this.selectedAlgorithm,algorithm_name:this.getAlgorithmName(this.selectedAlgorithm),model_id:this.selectedOpt1,model_name:this.getSelectedModelName(),model2_id:this.selectedOpt2,model2_name:this.getSelectedOptionName("add_opt2",this.selectedOpt2),model3_id:this.selectedOpt3,model3_name:this.getSelectedOptionName("add_opt3",this.selectedOpt3),format:this.selectedFormat,status:"waiting",progress:0,created_at:Date.now(),output_files:[],error:null,message:null,queue_count:null,current_order:null,phase:"queueing",download_file_name:null,download_bytes:0,download_total_bytes:null,download_speed_bps:0,download_percent:0};return this.tasks.push(n),this.saveActiveTasks(),this.sendDebugLog("INFO",`createTaskFromCurrentSelection success: hash=${o}`),this.render(),o}catch(o){return console.error("Failed to create task:",o),this.uploadFailed=!0,this.sendDebugLog("ERROR",`createTaskFromCurrentSelection failed: ${String(o)}`),this.setStatusBanner("error",l("home.createTaskFailed"),{showLogsShortcut:!0}),null}finally{this.isSubmittingTask=!1,this.submittingAction=null,this.render()}}async waitForTaskCompletion(e,s){var n;const i=this.getPollIntervalSeconds(),o=Date.now();for(;;){await this.pollTaskStatus(e);const a=this.tasks.find(u=>u.hash===e);if(!a)throw new Error(`Task ${e} not found`);if(this.isTerminalStatus(a.status))return a;if((Date.now()-o)/1e3>s)throw this.stopPolling(e),a.status="failed",a.error=`Timeout after ${s}s`,this.addToHistory(a,((n=this.config)==null?void 0:n.output_dir)||null),new Error(a.error);await new Promise(u=>setTimeout(u,i*1e3))}}getAlgorithmName(e){for(const s of this.algorithms){const i=s.algorithms.find(o=>o.id===e);if(i)return i.name}return`Algorithm ${e}`}getSelectedModelName(){if(this.selectedOpt1===null)return null;const e=this.algorithmDetails.get(this.selectedAlgorithm),s=e==null?void 0:e.fields.find(i=>i.name==="add_opt1");return s?s.options[String(this.selectedOpt1)]??null:null}normalizeTaskStatus(e){const s=(e||"").toLowerCase();return s==="done"||s==="success"||s==="completed"||s==="complete"||s==="finished"?"done":s==="failed"||s==="error"||s==="cancelled"||s==="canceled"||s==="timeout"?"failed":e}isTerminalStatus(e){const s=this.normalizeTaskStatus(e);return s==="done"||s==="failed"}isSuccessStatus(e){return this.normalizeTaskStatus(e)==="done"}getSelectedOptionName(e,s){if(s===null)return null;const i=this.algorithmDetails.get(this.selectedAlgorithm),o=i==null?void 0:i.fields.find(n=>n.name===e);return o?o.options[String(s)]??null:null}validateSelectedOptionsForCurrentAlgorithm(){const e=this.algorithmDetails.get(this.selectedAlgorithm);if(!e)return;const s=e.fields.find(n=>n.name==="add_opt1"),i=e.fields.find(n=>n.name==="add_opt2"),o=e.fields.find(n=>n.name==="add_opt3");this.selectedOpt1!==null&&(!s||!(String(this.selectedOpt1)in s.options))&&(this.selectedOpt1=null),this.selectedOpt2!==null&&(!i||!(String(this.selectedOpt2)in i.options))&&(this.selectedOpt2=null),this.selectedOpt3!==null&&(!o||!(String(this.selectedOpt3)in o.options))&&(this.selectedOpt3=null)}getPhaseFromStatus(e){const s=this.normalizeTaskStatus(e),i=s.toLowerCase();return i==="waiting"||i==="queued"||i==="queue"||i==="pending"||i==="distributing"?"queueing":i==="processing"||i==="running"||i==="working"||i==="merging"?"separating":s==="done"?"done":s==="failed"?"failed":"queueing"}formatBytes(e){if(e==null)return"-";if(e<1024)return`${e} B`;const s=e/1024;if(s<1024)return`${s.toFixed(1)} KB`;const i=s/1024;return i<1024?`${i.toFixed(1)} MB`:`${(i/1024).toFixed(2)} GB`}getDisplayFileName(e){const s=e.lastIndexOf("/"),i=e.lastIndexOf("\\"),o=Math.max(s,i);return(o>=0?e.slice(o+1):e)||e}startPolling(e){this.touchTaskPollingState(),Ye(this,e)}async pollTaskStatus(e){this.touchTaskPollingState(),await E(this,e)}stopPolling(e){this.touchTaskPollingState(),R(this,e)}touchTaskPollingState(){this.taskPollingIntervals,this.pollInFlightHashes}async downloadTask(e,s=null){await Xe(this,e,s)}async cancelDownload(e){await et(this,e)}deleteTask(e){const s=this.tasks.find(i=>i.hash===e);if(s){const i=s.phase||this.getPhaseFromStatus(s.status);if(!((s.status==="done"||s.status==="failed")&&i!=="downloading")){this.showTransientNotice("Only completed or failed tasks can be deleted.","warn",2600);return}}this.confirmAction("Delete this task from local list? This cannot be undone.")&&(this.stopPolling(e),this.tasks=this.tasks.filter(i=>i.hash!==e),this.saveActiveTasks(!0),this.requestTaskPanelsRefresh())}async openOutput(e){var o,n;const s=await this.resolveToAbsolutePath(e);this.addFrontendLog("INFO",`Open output requested: ${s}`),this.sendDebugLog("INFO",`openOutput requested: ${s}`);const i=/linux/i.test(navigator.userAgent);try{return await p("open_in_file_manager",{path:s}),this.addFrontendLog("INFO",`Open output succeeded: ${s}`),this.sendDebugLog("INFO",`openOutput succeeded via backend: ${s}`),!0}catch(a){if(this.addFrontendLog("WARN",`open_in_file_manager failed, trying fallback: ${String(a)}`),this.sendDebugLog("WARN",`openOutput backend open failed: ${String(a)}`),i){if((o=this.config)!=null&&o.output_dir)try{const u=await this.resolveToAbsolutePath(this.config.output_dir);return await p("open_in_file_manager",{path:u}),this.addFrontendLog("INFO",`Open output fallback succeeded: ${u}`),this.sendDebugLog("INFO",`openOutput fallback succeeded via backend: ${u}`),!0}catch{}return console.error("Failed to open output location:",a),this.sendDebugLog("ERROR",`openOutput failed on linux: ${String(a)}`),!1}try{return await W(s),this.addFrontendLog("INFO",`Reveal in dir succeeded: ${s}`),this.sendDebugLog("INFO",`openOutput succeeded via revealItemInDir: ${s}`),!0}catch(u){try{return await V(s),this.addFrontendLog("INFO",`openPath succeeded: ${s}`),this.sendDebugLog("INFO",`openOutput succeeded via openPath: ${s}`),!0}catch{}if((n=this.config)!=null&&n.output_dir)try{const h=await this.resolveToAbsolutePath(this.config.output_dir);return await p("open_in_file_manager",{path:h}),this.sendDebugLog("INFO",`openOutput fallback succeeded via backend: ${h}`),!0}catch{}return console.error("Failed to open output location:",a,u),this.sendDebugLog("ERROR",`openOutput failed after all fallbacks: ${String(a)} | ${String(u)}`),!1}}}async openHistoryOutput(e){var n;const s=this.taskHistory.find(a=>a.id===e);if(!s)return;const i=[];s.outputPath&&i.push(s.outputPath),s.outputFiles.length>0&&i.push(this.getParentPath(s.outputFiles[0])),(n=this.config)!=null&&n.output_dir&&i.push(this.config.output_dir);const o=[...new Set(i.filter(Boolean))];for(const a of o)if(await this.openOutput(a))return}getParentPath(e){const s=e.replace(/\\/g,"/"),i=s.lastIndexOf("/");return i<=0?e:s.slice(0,i)}async resolveToAbsolutePath(e){if(e.startsWith("/")||/^[a-zA-Z]:[\\/]/.test(e))return e;try{return await p("resolve_path",{path:e})}catch{return e}}saveCustomTheme(){if(!this.customThemeDraft)return;const e=this.customThemeDraft;document.documentElement.style.setProperty("--color-primary",e.primary),document.documentElement.style.setProperty("--color-primary-light",e.primary),document.documentElement.style.setProperty("--color-cta",e.primary),document.documentElement.style.setProperty("--color-bg-primary",e.bgPrimary),document.documentElement.style.setProperty("--color-text-primary",e.textPrimary),document.documentElement.setAttribute("data-theme","custom");const s=Ie("theme","custom"),i=F("mvsep_custom_theme",e);(!s||!i)&&console.error("Failed to persist custom theme"),this.addFrontendLog("INFO","Custom theme saved"),this.render()}getApiUrlByMirror(e){return e==="mirror"?"https://mirror.mvsep.com":"https://mvsep.com"}async chooseOutputDir(){try{await this.withUiAction("settings-choose-output",{runningMessage:l("settings.choosingFolder"),successMessage:l("settings.outputDirUpdated"),errorMessage:l("settings.chooseFolderFailed")},async()=>{const e=await I({directory:!0,multiple:!1});!e||!this.config||(this.config.output_dir=e)})}catch(e){console.error("Failed to choose output directory:",e)}finally{this.render()}}async testConnection(){var e,s;if(!(!((e=this.config)!=null&&e.token)||!((s=this.config)!=null&&s.api_url))){this.connectionStatus="testing",this.connectionStatusText=l("common.loading"),this.render();try{const i=await this.withUiAction("settings-test-connection",{runningMessage:l("settings.testing"),successMessage:l("settings.connectionSucceeded"),errorMessage:l("settings.connectionFailed")},async()=>{var o,n;return p("test_connection",{token:(o=this.config)==null?void 0:o.token,apiUrl:(n=this.config)==null?void 0:n.api_url})});if(i===null)return;this.connectionStatus=i?"success":"error",this.connectionStatusText=l(i?"settings.connected":"settings.disconnected")}catch{this.connectionStatus="error",this.connectionStatusText=l("settings.disconnected")}this.render()}}async saveSettings(){const e=document.getElementById("token-input"),s=document.getElementById("api-url-input"),i=document.getElementById("output-dir-input"),o=document.getElementById("poll-interval-input"),n=document.getElementById("proxy-mode-select"),a=document.getElementById("proxy-host-input"),u=document.getElementById("proxy-port-input"),h=document.getElementById("mirror-select"),f=document.getElementById("settings-output-format-select"),c=document.getElementById("algo-auto-refresh-days-input");if(this.config)try{await this.withUiAction("settings-save",{runningMessage:l("settings.saving"),successMessage:l("settings.saved"),errorMessage:l("settings.saveFailed")},async()=>{var b;this.config.token=((b=e==null?void 0:e.value)==null?void 0:b.trim())||this.defaultApiToken,this.config.api_url=(s==null?void 0:s.value)||"https://mvsep.com",this.config.output_dir=(i==null?void 0:i.value)||"./output";const m=parseInt((o==null?void 0:o.value)||"5",10),k=this.normalizePollInterval(m);this.config.poll_interval=k,this.config.proxy_mode=(n==null?void 0:n.value)||"system",this.config.proxy_host=(a==null?void 0:a.value)||"127.0.0.1",this.config.proxy_port=(u==null?void 0:u.value)||"7897",this.config.mirror=(h==null?void 0:h.value)||"main",this.config.output_format=f!=null&&f.value?parseInt(f.value,10):1;const w=parseInt((c==null?void 0:c.value)||String(this.defaultAlgorithmAutoRefreshDays),10),y=this.normalizeAlgorithmAutoRefreshDays(w);this.config.algorithm_auto_refresh_days=y,this.selectedFormat=this.config.output_format??this.selectedFormat,o&&(o.value=String(k)),m!==k&&this.showTransientNotice(`Poll interval adjusted to ${k}s (allowed: ${this.minPollIntervalSeconds}-${this.maxPollIntervalSeconds}s).`,"warn",3200),c&&(c.value=String(y)),await this.saveConfig(this.config),await this.loadFormats()}),this.showTransientNotice('Settings saved. Click "Fetch Latest Algorithm Info" to update local algorithm cache.',"info",3200)}catch(m){console.error("Failed to save settings:",m)}finally{this.render()}}toggleTokenVisibility(){this.isTokenVisible=!this.isTokenVisible,this.sendDebugLog("INFO",`token visibility toggled: ${this.isTokenVisible?"visible":"hidden"}`),this.render()}showApiTokenHelp(){this.showTransientNotice(l("settings.apiKeyHelpText"),"info",6200),this.setStatusBanner("running",l("settings.apiKeyHelpShown"),{autoHideMs:2600,showLogsShortcut:!1}),this.sendDebugLog("INFO","api token help requested")}normalizePollInterval(e){if(!Number.isFinite(e))return 5;const s=Math.round(e);return Math.min(this.maxPollIntervalSeconds,Math.max(this.minPollIntervalSeconds,s))}getPollIntervalSeconds(){var e;return this.normalizePollInterval(((e=this.config)==null?void 0:e.poll_interval)??5)}normalizeAlgorithmAutoRefreshDays(e){if(!Number.isFinite(e))return this.defaultAlgorithmAutoRefreshDays;const s=Math.round(e);return Math.min(365,Math.max(1,s))}parseCacheUpdatedAtToMs(e){if(!e)return null;const s=Number(e);if(Number.isFinite(s)&&s>0)return s*1e3;const i=Date.parse(e);return Number.isFinite(i)&&i>0?i:null}shouldAutoRefreshAlgorithmCache(){var o,n;if(!((o=this.config)!=null&&o.token)||!((n=this.config)!=null&&n.api_url))return!1;const e=this.normalizeAlgorithmAutoRefreshDays(this.config.algorithm_auto_refresh_days),s=this.parseCacheUpdatedAtToMs(this.lastAlgorithmCacheUpdatedAt);if(s===null)return!0;const i=e*24*60*60*1e3;return Date.now()-s>=i}confirmAction(e){return window.confirm(e)}render(){this.renderPending=!0,!this.renderScheduled&&(this.renderScheduled=!0,window.requestAnimationFrame(()=>{this.renderScheduled=!1,this.renderPending&&(this.renderPending=!1,this.performRender(),this.renderPending&&this.render())}))}performRender(){const e=document.getElementById("app");if(!e)return;if(this.renderInProgress){this.droppedRenderCount+=1,this.renderPending=!0;return}this.renderInProgress=!0;const s=++this.renderSeq,i=performance.now();try{const o=document.activeElement,n=(o==null?void 0:o.id)||null,a=o instanceof HTMLInputElement||o instanceof HTMLTextAreaElement,u=a?o.selectionStart:null,h=a?o.selectionEnd:null;(!this.isShellMounted||!document.getElementById("main-content"))&&(e.innerHTML=`
          <div class="flex h-screen">
            <div id="sidebar-root"></div>
            <div id="main-content" class="flex-1 md:ml-56 p-4 md:p-6 pb-24 md:pb-6 overflow-y-auto"></div>
          </div>
        `,this.isShellMounted=!0);const f=document.getElementById("sidebar-root");if(f){const y=`${this.currentPage}|${H()}`;this.lastSidebarRenderKey!==y&&(f.innerHTML=this.renderSidebar(),this.lastSidebarRenderKey=y)}const c=this.currentPage==="logs"?document.getElementById("log-container"):null;if(c){const y=c.scrollHeight-c.scrollTop-c.clientHeight;this.logStickToBottom=y<=32}const m=document.getElementById("main-content");if(!m)return;const k=this.lastRenderedPage===this.currentPage,w=m.scrollTop;if(m.innerHTML=`
        ${this.transientNotice?`
          <div class="mb-4 px-3 py-2 rounded-lg border ${this.transientNotice.level==="warn"?"bg-yellow-50 border-yellow-200 text-yellow-800":"bg-blue-50 border-blue-200 text-blue-800"} text-sm">
            ${this.escapeHtml(this.transientNotice.message)}
          </div>
        `:""}
        ${this.renderHeader()}
        ${this.renderStatusBanner()}
        ${this.renderPage()}
      `,k?m.scrollTop=w:m.scrollTop=0,n&&a){const y=document.getElementById(n);if(y){try{y.focus({preventScroll:!0})}catch{try{y.focus()}catch{}}if(u!==null&&h!==null)try{const b=y instanceof HTMLInputElement?y.type:"text",d=["text","search","url","tel","password"];(!(y instanceof HTMLInputElement)||d.includes(b))&&y.setSelectionRange(u,h)}catch{}}}if(this.lastRenderedPage=this.currentPage,this.currentPage==="logs"){const y=document.getElementById("log-container");if(y){const b=this.getFilteredLogs().length,d=this.lastRenderedLogCountByType[this.currentLogType]??0;b>d&&this.logStickToBottom&&(y.scrollTop=y.scrollHeight),this.lastRenderedLogCountByType[this.currentLogType]=b}}}catch(o){console.error("Render failed:",o),this.sendDebugLog("ERROR",`render failed: ${String(o)} (page=${this.currentPage}, renderId=${s})`),e.innerHTML=`
        <div class="p-6">
          <div class="card">
            <h3 class="font-semibold text-red-500 mb-2">Render Error</h3>
            <p class="text-sm text-text-muted">UI render failed. Check frontend logs for details.</p>
          </div>
        </div>
      `}finally{const o=performance.now()-i;o>120?this.sendDebugLog("WARN",`slow render: id=${s}, page=${this.currentPage}, cost_ms=${o.toFixed(1)}, dropped=${this.droppedRenderCount}`):this.droppedRenderCount>0&&this.sendDebugLog("DEBUG",`render recovered: id=${s}, page=${this.currentPage}, dropped=${this.droppedRenderCount}`),this.droppedRenderCount=0,this.renderInProgress=!1}}renderSidebar(){const e=[{id:"home",icon:"🏠",label:l("nav.home")},{id:"tasks",icon:"🎵",label:l("nav.tasks")},{id:"algorithms",icon:"🔍",label:l("nav.algorithms")},{id:"settings",icon:"⚙️",label:l("nav.settings")},{id:"appearance",icon:"🎨",label:l("nav.appearance")},{id:"logs",icon:"📋",label:l("nav.logs")},{id:"about",icon:"ℹ️",label:l("nav.about")}];return`
      <aside class="sidebar">
        <div class="p-4 border-b border-border">
          <h1 class="text-xl font-bold text-primary">MVSEP</h1>
          <p class="text-xs text-text-muted">${l("app.title")}</p>
        </div>
        <nav class="flex-1 py-2">
          ${e.map(s=>`
            <button
              type="button"
              class="sidebar-item ${this.currentPage===s.id?"active":""}"
              data-nav="${s.id}"
              aria-current="${this.currentPage===s.id?"page":"false"}"
            >
              <span>${s.icon}</span>
              <span>${s.label}</span>
            </button>
          `).join("")}
        </nav>
      </aside>
    `}renderHeader(){const e=Le(),s=H();return`
      <div class="flex items-center justify-between mb-6">
        <div class="flex items-center gap-4">
          <h2 class="text-2xl font-bold text-text-primary">${l(`nav.${this.currentPage}`)}</h2>
          <div id="queue-info-slot">${this.renderQueueInfoBadge()}</div>
        </div>
        <div class="flex items-center gap-3">
          <select class="select w-28" id="locale-select" ${this.isLoadingAlgorithmDetails?"disabled":""}>
            ${e.map(i=>`
              <option value="${i.code}" ${s===i.code?"selected":""}>${i.nativeName}</option>
            `).join("")}
          </select>
        </div>
      </div>
    `}renderStatusBanner(){return this.statusBanner?`
      <div class="mb-4 px-3 py-2 rounded-lg border ${this.statusBanner.phase==="error"?"bg-red-50 border-red-200 text-red-800":this.statusBanner.phase==="success"?"bg-green-50 border-green-200 text-green-800":"bg-blue-50 border-blue-200 text-blue-800"} text-sm flex items-center justify-between gap-2">
        <span>${this.escapeHtml(this.statusBanner.message)}</span>
        ${this.statusBanner.showLogsShortcut?`
          <button class="btn btn-secondary text-xs px-2 py-1" type="button" data-action="open-logs-page">
            ${l("logs.title")}
          </button>
        `:""}
      </div>
    `:""}renderPage(){switch(this.currentPage){case"home":return this.renderHomePage();case"tasks":return this.renderTasksPage();case"algorithms":return this.renderAlgorithmsPage();case"settings":return this.renderSettingsPage();case"appearance":return this.renderAppearancePage();case"logs":return this.renderLogsPage();case"about":return this.renderAboutPage();default:return this.renderHomePage()}}renderHomeCurrentTasksSection(){const e=this.getInProgressTasks();return Ee({currentTasks:e,renderTaskCard:s=>this.renderTaskCard(s),t:l})}renderHomeRecentHistorySection(){const e=this.taskHistory.slice(0,3);return Re({recentHistory:e,renderHistoryCard:s=>this.renderHistoryCard(s),t:l})}renderHomePage(){const e=this.selectedFile?this.escapeHtml(this.selectedFile.split("/").pop()||this.selectedFile):"",s=this.algorithms.flatMap(a=>a.algorithms).map(a=>`
        <option value="${a.id}" ${a.id===this.selectedAlgorithm?"selected":""}>
          ${this.escapeHtml(a.name)}
        </option>
      `).join(""),i=this.presets.length>0?`
        <div>
          <label class="block text-sm text-text-secondary mb-1">${l("home.presetLoadLabel")}</label>
          <div class="flex gap-2">
            <select class="select" id="home-preset-select">
              ${this.presets.map(a=>`
                <option value="${a.id}" ${a.id===this.selectedHomePresetId?"selected":""}>${this.escapeHtml(a.name)}</option>
              `).join("")}
            </select>
            <button class="btn btn-secondary whitespace-nowrap" data-action="apply-home-preset">${l("common.load")}</button>
          </div>
        </div>
      `:"",o=this.formats.map(a=>`
      <option value="${a.id}" ${a.id===this.selectedFormat?"selected":""}>
        ${this.escapeHtml(a.name)}
      </option>
    `).join(""),n=`
      <div class="text-sm px-3 py-2 rounded-lg bg-bg-primary border border-border text-text-secondary">
        <p class="mb-1">${l("home.uploading")}: ${this.escapeHtml(this.uploadFileName||"audio")}</p>
        <div class="progress-bar mb-1">
          <div class="progress-bar-fill" style="width: ${Math.max(0,Math.min(100,this.uploadPercent||0))}%"></div>
        </div>
        <p>${(this.uploadPercent||0).toFixed(1)}% · ${this.formatBytes(this.uploadBytes)} / ${this.formatBytes(this.uploadTotalBytes||null)} · ${this.formatBytes(this.uploadSpeedBps)}/s</p>
        ${this.uploadFailed?`<p class="text-red-500 mt-1">${l("home.uploadFailed")}</p>`:""}
      </div>
    `;return Ue({isInitialLoading:this.isInitialLoading,selectedFile:this.selectedFile,selectedFileLabelEscaped:e,algorithmSelectOptionsHtml:s,algorithmOptionsHtml:this.renderAlgorithmOptions(),presetBlockHtml:i,formatOptionsHtml:o,isDemo:this.isDemo,runTimeoutSeconds:this.runTimeoutSeconds,isSubmittingTask:this.isSubmittingTask,submittingAction:this.submittingAction,uploadInfoHtml:n,currentTasksSectionHtml:this.renderHomeCurrentTasksSection(),recentHistorySectionHtml:this.renderHomeRecentHistorySection(),t:l})}renderAlgorithmOptions(){const e=this.algorithmDetails.get(this.selectedAlgorithm);if(!e)return`
        <div class="text-sm text-text-muted flex items-center justify-between">
          <span>${l("algorithm.loadingParams")}</span>
          <button class="btn btn-secondary text-sm" data-action="refresh-algo-list" ${this.isActionRunning("algo-refresh-list")?"disabled":""}>
            ${this.isActionRunning("algo-refresh-list")?l("common.loading"):l("algorithm.refreshList")}
          </button>
        </div>
      `;if(e.fields.length===0)return`
        <div class="text-sm text-text-muted flex items-center justify-between">
          <span>${l("algorithm.noAddOpt")}</span>
          <button class="btn btn-secondary text-sm" data-action="refresh-algo-list" ${this.isActionRunning("algo-refresh-list")?"disabled":""}>
            ${this.isActionRunning("algo-refresh-list")?l("common.loading"):l("algorithm.refreshList")}
          </button>
        </div>
      `;let s="";for(const i of e.fields){const o=i.name.replace("add_",""),n=Object.entries(i.options),a=i.text||i.name;if(n.length===0)continue;const u=o==="opt1"?this.selectedOpt1:o==="opt2"?this.selectedOpt2:this.selectedOpt3;s+=`
        <div>
          <label class="block text-sm text-text-secondary mb-1">
            ${this.escapeHtml(a)} (--${this.escapeHtml(i.name)})
            ${o==="opt1"?'<span class="text-red-500">*</span>':""}
          </label>
          <select class="select" id="${o}-select">
            <option value="">-- Select --</option>
            ${n.map(([h,f])=>`
              <option value="${this.escapeHtml(h)}" ${u===parseInt(h,10)?"selected":""}>
                ${this.escapeHtml(h)}: ${this.escapeHtml(String(f))}
              </option>
            `).join("")}
          </select>
        </div>
      `}return`
      ${s}
      <button class="btn btn-secondary text-sm" data-action="refresh-algo-list" ${this.isActionRunning("algo-refresh-list")?"disabled":""}>
        ${this.isActionRunning("algo-refresh-list")?l("common.loading"):l("algorithm.refreshList")}
      </button>
    `}renderTaskCard(e){var f,c;const s=e.phase||this.getPhaseFromStatus(e.status)||"queueing",i=`status-${s==="queueing"?"waiting":s==="separating"?"processing":s}`,o=l(s==="queueing"?"task.status.queueing":s==="separating"?"task.status.separating":s==="downloading"?"task.status.downloading":`task.status.${e.status}`),n=this.expandedTaskHashes.has(e.hash),a=((f=this.formats.find(m=>m.id===e.format))==null?void 0:f.name)||`Format ${e.format}`,u=this.selectedDownloadFileIndexByHash.get(e.hash)??null,h=this.cancellingDownloadHashes.has(e.hash);return Ne({task:e,phase:s,statusClass:i,statusText:o,isExpanded:n,formatName:a,selectedDownloadIndex:u,isCancellingDownload:h,queueQueuedFallback:(c=this.queueInfo)==null?void 0:c.queued,escapeHtml:m=>this.escapeHtml(m),formatBytes:m=>this.formatBytes(m),getDisplayFileName:m=>this.getDisplayFileName(m),t:l})}renderTasksFilterBar(){const e=this.getInProgressTasks(),s=this.getCompletedTasks();return De({taskViewFilter:this.taskViewFilter,inProgressCount:e.length,completedCount:s.length,historyCount:this.taskHistory.length,historySearchQueryEscaped:this.escapeHtml(this.historySearchQuery),historySortOrder:this.historySortOrder,t:l})}renderTasksViewContent(){const e=this.getInProgressTasks(),s=this.getCompletedTasks(),i=this.getPaginatedHistory(),o=this.getFilteredHistory(),n=Math.ceil(o.length/this.historyPageSize);return Ce({taskViewFilter:this.taskViewFilter,inProgressTasks:e,completedTasks:s,historyRecords:i,historyPage:this.historyPage,totalPages:n,renderTaskCard:a=>this.renderTaskCard(a),renderHistoryCard:a=>this.renderHistoryCard(a),t:l})}renderTasksPage(){return Oe({filterBarHtml:this.renderTasksFilterBar(),viewContentHtml:this.renderTasksViewContent(),t:l})}renderHistoryCard(e){const s=e.status==="done"?"status-done":"status-failed",i=e.status==="done"?l("task.status.done"):l("task.status.failed"),o=e.completedAt?new Date(e.completedAt).toLocaleString():"-";return Be({record:e,statusClass:s,statusText:i,completedDate:o,escapeHtml:n=>this.escapeHtml(n),t:l})}renderAlgorithmsPage(){return Ve({groups:this.algorithmSearchResults,algorithmSearchQueryEscaped:this.escapeHtml(this.algorithmSearchQuery),presets:this.presets,presetNameInputEscaped:this.escapeHtml(this.presetNameInput),expandedAlgorithmId:this.expandedAlgorithmId,renderAlgorithmDetailsSection:e=>this.renderAlgorithmDetailsSection(e),escapeHtml:e=>this.escapeHtml(e),isFetchingLatest:this.isActionRunning("algo-fetch-latest"),isRefreshingList:this.isActionRunning("algo-refresh-list"),t:l})}renderAlgorithmDetailsSection(e){return je({details:this.algorithmDetails.get(e),escapeHtml:s=>this.escapeHtml(s),t:l})}renderSettingsPage(){return Me({config:this.config,formats:this.formats,connectionStatus:this.connectionStatus,connectionStatusText:this.connectionStatusText,algorithmCachePath:this.algorithmCachePath,isTokenVisible:this.isTokenVisible,isTestingConnection:this.isActionRunning("settings-test-connection"),isChoosingOutputDir:this.isActionRunning("settings-choose-output"),isSavingSettings:this.isActionRunning("settings-save"),t:l})}renderAppearancePage(){const e=_(),s=document.documentElement.getAttribute("data-theme"),i=this.customThemeDraft||{primary:"#0891B2",bgPrimary:"#ECFEFF",textPrimary:"#164E63"};return qe({themes:e,currentThemeId:s,draft:i,t:l})}renderLogsPage(){return He({currentLogType:this.currentLogType,logFilterLevel:this.logFilterLevel,logSearchQuery:this.logSearchQuery,frontendLogs:this.frontendLogs,backendLogs:this.backendLogs,escapeHtml:e=>this.escapeHtml(e),t:l})}escapeHtml(e){const s=document.createElement("div");return s.textContent=e,s.innerHTML}getFilteredLogs(){const e=this.currentLogType==="frontend"?this.frontendLogs:this.backendLogs;return O(e,this.logFilterLevel,this.logSearchQuery)}async switchLogType(e){this.currentLogType=e,e==="backend"&&await this.loadBackendLogs(),this.render()}clearLogs(){this.currentLogType==="frontend"?this.frontendLogs=[]:this.backendLogs=[],this.render()}exportLogs(){const s=this.getFilteredLogs().map(a=>`[${a.timestamp}] [${a.level}] ${a.message}`).join(`
`),i=new Blob([s],{type:"text/plain"}),o=URL.createObjectURL(i),n=document.createElement("a");n.href=o,n.download=`mvsep-${this.currentLogType}-logs-${new Date().toISOString().slice(0,10)}.log`,n.click(),URL.revokeObjectURL(o)}copyLogs(){const s=this.getFilteredLogs().map(i=>`[${i.timestamp}] [${i.level}] ${i.message}`).join(`
`);navigator.clipboard.writeText(s).then(()=>{this.addFrontendLog("INFO","Logs copied to clipboard"),this.render()})}renderAboutPage(){return _e(l)}}new tt;
