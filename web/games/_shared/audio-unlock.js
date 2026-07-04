// Web Audio unlock for WebKit (iOS + macOS Safari). Two distinct problems:
//
// 1) Suspended context. Bevy/cpal creates its AudioContext eagerly at startup,
//    before any user gesture, so it comes up "suspended". Chrome and Firefox
//    auto-resume it once the user interacts and a source node start()s, so no
//    shim is needed there. WebKit does NOT auto-resume on start(): it needs an
//    explicit resume() inside a real user-gesture handler. Bevy does not expose
//    its AudioContext from Rust, so we wrap the constructor to capture every
//    context it builds and resume them all on the first gesture.
//
// 2) iOS ringer channel (WebKit bug 237322). On iOS, Web Audio output goes to
//    the RINGER channel, which the physical mute switch silences even at full
//    media volume -- so after (1) the tab shows a "playing audio" indicator but
//    is still inaudible when the switch is on Silent. HTML5 <audio> elements
//    play on the MEDIA channel, which ignores the switch, and while one is
//    playing iOS promotes the whole session (Web Audio included) to media. So
//    on iOS we also start a continuous looping, inaudible <audio> on the first
//    gesture. Refs: swevans/unmute, feross/unmute-ios-audio.
//
// This is the single shared copy for every web game. Each game's trunk
// index.html loads it with a plain, non-module, non-defer
// `<script src="audio-unlock.js"></script>` in <head> (which runs before
// trunk's injected `<script type="module">` wasm loader, so the constructor
// wrap is installed before Bevy builds its context) and stages it into the
// dist with `<link data-trunk rel="copy-file" href="../_shared/audio-unlock.js" />`.
// See docs/wasm-web-builds.md. Do NOT re-inline this per game: the copies
// drifted once (07_orbit/08_dropzone kept an older version without the iOS
// media-channel fix and went silent on iPhone), which is what this shared file
// prevents.
(function () {
  var Native = window.AudioContext || window.webkitAudioContext;
  if (!Native) return;

  // Every AudioContext Bevy/cpal builds gets recorded here.
  var contexts = [];
  function Wrapped() {
    var ctx = new (Function.prototype.bind.apply(
      Native,
      [null].concat([].slice.call(arguments))
    ))();
    contexts.push(ctx);
    return ctx;
  }
  Wrapped.prototype = Native.prototype;
  window.AudioContext = Wrapped;
  if (window.webkitAudioContext) window.webkitAudioContext = Wrapped;

  // The ringer-channel workaround is only needed on iOS (incl. iPadOS, which
  // reports as Mac but has touch). Elsewhere a looping <audio> would just add a
  // needless "now playing" widget, so gate on iOS.
  var ua = navigator.userAgent || '';
  var isIOS =
    /iP(hone|ad|od)/.test(ua) ||
    (/Mac/.test(ua) && navigator.maxTouchPoints > 1);

  // A few ms of silence; loops forever, inaudible. Generated as a mono 8 kHz
  // 16-bit PCM WAV of zero samples (see task 20260703-212303).
  var SILENCE =
    'data:audio/wav;base64,UklGRkQDAABXQVZFZm10IBAAAAABAAEAQB8AAIA+AAACABAAZGF0YSADAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==';
  var silent = null;
  function playSilentTrack() {
    if (!isIOS) return;
    if (!silent) {
      silent = document.createElement('audio');
      silent.src = SILENCE;
      silent.loop = true;
      silent.setAttribute('playsinline', '');
      silent.preload = 'auto';
    }
    // Must be called inside a user gesture on iOS; ignore rejections.
    var p = silent.play();
    if (p && p.catch) p.catch(function () {});
  }

  var events = ['pointerdown', 'touchend', 'mousedown', 'keydown'];
  function detachIfRunning() {
    // On iOS keep listening: the silent track can be interrupted and needs
    // re-arming from a later gesture; the resume()/kick below is a cheap no-op
    // once already running.
    if (isIOS) return;
    var running =
      contexts.length > 0 &&
      contexts.every(function (c) {
        return c.state === 'running';
      });
    if (!running) return;
    events.forEach(function (type) {
      document.removeEventListener(type, unlock, true);
    });
  }
  function unlock() {
    playSilentTrack();
    contexts.forEach(function (ctx) {
      if (ctx.state === 'suspended' && ctx.resume) {
        ctx.resume().then(detachIfRunning, function () {});
      }
      // WebKit's stricter unlock wants an actual node start inside the gesture;
      // a 1-sample silent buffer is inaudible and does it.
      try {
        var src = ctx.createBufferSource();
        src.buffer = ctx.createBuffer(1, 1, ctx.sampleRate);
        src.connect(ctx.destination);
        src.start(0);
      } catch (e) {}
    });
    detachIfRunning();
  }
  events.forEach(function (type) {
    document.addEventListener(type, unlock, {
      capture: true,
      passive: true,
    });
  });

  // Pause the silent track when backgrounded (so iOS does not keep a media
  // widget alive / drain battery) and resume it on return.
  document.addEventListener('visibilitychange', function () {
    if (!silent) return;
    if (document.hidden) {
      silent.pause();
    } else {
      var p = silent.play();
      if (p && p.catch) p.catch(function () {});
    }
  });
})();
