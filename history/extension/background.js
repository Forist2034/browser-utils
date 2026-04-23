let port = browser.runtime.connectNative("browser_utils_history_host");

browser.storage.sync.get("root").then((data) => {
  console.log(data);
  port.postMessage({ init: { root: data.root } });
});

port.onMessage.addListener((err) => {
  console.error(err);
  browser.notifications.create("error", {
    type: "basic",
    title: "History host error",
    message: err,
  });
});
port.onDisconnect.addListener((port) => {
  port.postMessage("disconnect");
});

browser.history.onVisited.addListener((item) => {
  port.postMessage({ on_visit: item });
});

browser.history.onTitleChanged.addListener((item) => {
  port.postMessage({ on_title_update: item });
});
