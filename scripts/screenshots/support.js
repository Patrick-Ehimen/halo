// Static renderer for .dc.html design components: evaluates the component's
// renderVals() once and expands {{expr}}, <sc-for>, <sc-if>. Screenshot use only.
window.DCLogic = class {
  constructor(props) {
    this.props = props || {};
    this.state = {};
  }
  setState(s) {
    Object.assign(this.state, s);
  }
};
function ev(expr, scope) {
  try {
    return new Function("s", "with(s){return (" + expr + ")}")(scope);
  } catch (e) {
    console.warn("expr", expr, e);
    return "";
  }
}
const RE = /\{\{([\s\S]+?)\}\}/g;
function interp(str, scope) {
  return str.replace(RE, (_, e) => {
    const v = ev(e, scope);
    return v == null ? "" : String(v);
  });
}
function single(str) {
  const m = str.trim().match(/^\{\{([\s\S]+?)\}\}$/);
  return m ? m[1] : null;
}
function proc(node, scope) {
  for (const child of Array.from(node.childNodes)) {
    if (child.nodeType === 3) {
      if (RE.test(child.data)) {
        RE.lastIndex = 0;
        child.data = interp(child.data, scope);
      }
      RE.lastIndex = 0;
      continue;
    }
    if (child.nodeType !== 1) continue;
    const tag = child.localName;
    if (tag === "sc-for") {
      const list = ev(single(child.getAttribute("list")), scope) || [];
      const as = child.getAttribute("as") || "item";
      const frag = document.createDocumentFragment();
      list.forEach((item, i) => {
        const wrap = child.cloneNode(true);
        proc(wrap, Object.assign(Object.create(scope), { [as]: item, index: i }));
        while (wrap.firstChild) frag.appendChild(wrap.firstChild);
      });
      child.replaceWith(frag);
      continue;
    }
    if (tag === "sc-if") {
      const v = ev(single(child.getAttribute("value")), scope);
      if (v) {
        proc(child, scope);
        child.replaceWith(...Array.from(child.childNodes));
      } else child.remove();
      continue;
    }
    for (const a of Array.from(child.attributes)) {
      if (!a.value.includes("{{")) continue;
      if (/^on/i.test(a.name)) {
        child.removeAttribute(a.name);
        continue;
      }
      const one = single(a.value);
      if (one !== null) {
        const v = ev(one, scope);
        if (v === false || v == null) child.removeAttribute(a.name);
        else child.setAttribute(a.name, v === true ? "" : String(v));
      } else child.setAttribute(a.name, interp(a.value, scope));
    }
    proc(child, scope);
  }
}
document.addEventListener("DOMContentLoaded", () => {
  const script = document.querySelector("script[data-dc-script]");
  const root = document.querySelector("x-dc");
  const helmet = root.querySelector("helmet");
  if (helmet) {
    for (const n of Array.from(helmet.childNodes)) document.head.appendChild(n);
    helmet.remove();
  }
  let scope = {};
  if (script) {
    const decl = JSON.parse(script.dataset.props || "{}");
    const props = {};
    for (const [k, v] of Object.entries(decl))
      if (k[0] !== "$") props[k] = v && "default" in v ? v.default : v;
    const C = new Function("DCLogic", script.textContent + "\nreturn Component;")(window.DCLogic);
    const inst = new C(props);
    scope = inst.renderVals ? inst.renderVals() : {};
    script.remove();
  }
  proc(root, scope);
  document.documentElement.dataset.ready = "1";
});
