document.addEventListener("DOMContentLoaded", () => {
  browser.storage.local.get("root").then((v) => {
    document.getElementById("store-root").value = v.root || "";
  });
});
document.querySelector("form").addEventListener("submit", () => {
  browser.storage.local.set({
    root: document.getElementById("store-root").value,
  });
});
