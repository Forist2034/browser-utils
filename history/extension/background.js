let port = browser.runtime.connectNative("browser_utils_history_host");

browser.storage.local.get("root").then((data) => {
  browser.runtime.getBrowserInfo().then((info) => {
    const val = {
      root: data.root,
      browser: info,
    };
    console.log(val);
    port.postMessage({ init: val });
  });
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

function addEvent(msg) {
  port.postMessage({ event: msg });
}

browser.history.onVisited.addListener((item) =>
  addEvent({ history: { visit: item } })
);
browser.history.onTitleChanged.addListener((item) =>
  addEvent({ history: { title_update: item } })
);

function addTabEvent(ev) {
  addEvent({ tab: ev });
}
browser.tabs.onCreated.addListener((tab) => addTabEvent({ created: tab }));
browser.tabs.onUpdated.addListener((_0, _1, tab) =>
  addTabEvent({ updated: tab })
);
browser.tabs.onMoved.addListener((tabId, moveInfo) =>
  addTabEvent({ moved: { tab_id: tabId, move_info: moveInfo } })
);
browser.tabs.onAttached.addListener((tabId, attachInfo) =>
  addTabEvent({ attached: { tab_id: tabId, attach_info: attachInfo } })
);
browser.tabs.onDetached.addListener((tabId, detachInfo) =>
  addTabEvent({ detached: { tab_id: tabId, detach_info: detachInfo } })
);
browser.tabs.onActivated.addListener((info) =>
  addTabEvent({ activated: info })
);
browser.tabs.onReplaced.addListener((added, removed) =>
  addTabEvent({ replaced: { added_tab_id: added, removed_tab_id: removed } })
);
browser.tabs.onRemoved.addListener((tabId, removeInfo) =>
  addTabEvent({ removed: { tab_id: tabId, remove_info: removeInfo } })
);

function addTabGroupEvent(ev) {
  addEvent({ tab_group: ev });
}
browser.tabGroups.onCreated.addListener((group) =>
  addTabGroupEvent({ created: group })
);
browser.tabGroups.onMoved.addListener((group) =>
  addTabGroupEvent({ moved: group })
);
browser.tabGroups.onUpdated.addListener((group) =>
  addTabGroupEvent({ updated: group })
);
browser.tabGroups.onRemoved.addListener((group, removeInfo) =>
  addTabGroupEvent({ removed: { group, remove_info: removeInfo } })
);

function addNavigationEvent(ev) {
  addEvent({ navigation: ev });
}
browser.webNavigation.onCreatedNavigationTarget.addListener((details) =>
  addNavigationEvent({ created_navigation_target: details })
);
browser.webNavigation.onCommitted.addListener((details) =>
  addNavigationEvent({ committed: details })
);
browser.webNavigation.onCompleted.addListener((details) =>
  addNavigationEvent({ completed: details })
);
browser.webNavigation.onReferenceFragmentUpdated.addListener((details) =>
  addNavigationEvent({ reference_fragment_updated: details })
);
browser.webNavigation.onHistoryStateUpdated.addListener((details) =>
  addNavigationEvent({ history_state_updated: details })
);
browser.webNavigation.onErrorOccurred.addListener((details) =>
  addNavigationEvent({ error_occurred: details })
);
browser.webNavigation.onTabReplaced.addListener((details) =>
  addNavigationEvent({ tab_replaced: details })
);

browser.windows.onCreated.addListener((window) =>
  addEvent({ window: { created: window } })
);
browser.windows.onRemoved.addListener((windowId) =>
  addEvent({ window: { removed: windowId } })
);
