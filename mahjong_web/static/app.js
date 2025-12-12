// ====== Tile definitions ======
// internal tile code: "1m".."9m", "1p".."9p", "1s".."9s", honors: E,S,W,N,P,F,C, red: "0m/0p/0s" (=aka 5)
const SUITS = ["m","p","s"];
const HONORS = ["E","S","W","N","P","F","C"];
const HONOR_LABEL = { E:"東", S:"南", W:"西", N:"北", P:"白", F:"發", C:"中" };

function codeToLabel(code){
  if (code === "") return "";
  if (code[0] === "0") {
    const suit = code[1];
    return suit==="m"?"赤五萬":suit==="p"?"赤五筒":"赤五索";
  }
  if (HONORS.includes(code)) return HONOR_LABEL[code];
  const n = Number(code[0]);
  const suit = code[1];
  const numKanji = "一二三四五六七八九"[n-1];
  const suitKanji = suit==="m"?"萬":suit==="p"?"筒":"索";
  return `${numKanji}${suitKanji}`;
}

function labelToCode(label){
  // expects labels like "一萬" / "東" / "赤五萬"
  if (!label) return "";
  if (label.startsWith("赤五")) {
    if (label.includes("萬")) return "0m";
    if (label.includes("筒")) return "0p";
    return "0s";
  }
  if (label.length === 1) {
    // honor
    const k = Object.entries(HONOR_LABEL).find(([,v])=>v===label);
    return k ? k[0] : "";
  }
  const numMap = { "一":1,"二":2,"三":3,"四":4,"五":5,"六":6,"七":7,"八":8,"九":9 };
  const n = numMap[label[0]];
  const suitChar = label[1];
  const suit = suitChar==="萬"?"m":suitChar==="筒"?"p":suitChar==="索"?"s":"";
  return suit ? `${n}${suit}` : "";
}

function allPaletteCodes(){
  const v = [];
  for (const s of SUITS) for (let n=1;n<=9;n++) v.push(`${n}${s}`);
  for (const h of HONORS) v.push(h);
  // red fives selectable via toggle, not always separate
  return v;
}

// ====== SVG ======
function svgTile(label){
  const w=46,h=64;
  const isRed = label.startsWith("赤");
  const fill = isRed ? "#fff2f2" : "#fafafa";
  const text = label;
  return `
    <svg width="${w}" height="${h}" viewBox="0 0 ${w} ${h}" xmlns="http://www.w3.org/2000/svg">
      <rect x="1" y="1" width="${w-2}" height="${h-2}" rx="10" fill="white" stroke="#2a2c33"/>
      <rect x="6" y="6" width="${w-12}" height="${h-12}" rx="8" fill="${fill}" stroke="#d9dde3"/>
      <text x="${w/2}" y="${h/2}" text-anchor="middle" dominant-baseline="central"
            font-size="14" font-family="system-ui, -apple-system, Segoe UI, sans-serif"
            fill="${isRed ? "#d11" : "#111"}">${text}</text>
    </svg>`;
}

// ====== State ======
const state = {
  // selection target: { kind: "hand"|"win"|"dora"|"kanDora"|"ura"|"kanUra", index }
  target: { kind: "hand", index: 0 },

  roundWind: "E",
  seatWind: "E",
  kyotaku: 0,
  honba: 0,

  winType: "RON", // RON/TSUMO
  dealer: true,   // true=親

  riichi: "NONE", // NONE/RIICHI/DOUBLE
  ippatsu: false,
  rinshan: false,
  chankan: false,
  haitei: "NONE", // NONE/HAITEI/HOUTEI
  tenhou: "NONE", // NONE/TENHOU/CHIHOU

  akaOn: false,

  hand: Array(13).fill(""),
  win: "",

  doraIndicators: Array(5).fill(""), // fixed 5
  kanDoraIndicators: [],             // length = kanCount (0..4)

  uraIndicators: [],                 // shown only when riichi!=NONE. length = countFilled(doraIndicators)
  kanUraIndicators: [],              // length = kanCount

  meldMode: "NORMAL", // NORMAL/CHI/PON/MINKAN/ANKAN
  meldDraft: [],      // tile codes being selected for current meld
  melds: [],          // {type, tiles[]}
};

const el = (id)=>document.getElementById(id);

// ====== UI builders ======
function buildSeg(containerId, options, getVal, setVal){
  const c = el(containerId);
  c.innerHTML = "";
  for (const opt of options){
    const b = document.createElement("button");
    b.textContent = opt.label;
    if (getVal() === opt.value) b.classList.add("active");
    b.addEventListener("click", ()=>{ setVal(opt.value); renderAll(); });
    c.appendChild(b);
  }
}

function buildCounter(containerId, getVal, setVal){
  const c = el(containerId);
  c.innerHTML = "";
  const minus = document.createElement("button"); minus.textContent="−";
  const reset = document.createElement("button"); reset.textContent="↻"; reset.className="reset";
  const plus  = document.createElement("button"); plus.textContent="+";
  const val   = document.createElement("div"); val.className="val"; val.textContent=String(getVal());

  minus.addEventListener("click", ()=>{ setVal(Math.max(0, getVal()-1)); renderAll(); });
  plus.addEventListener("click",  ()=>{ setVal(getVal()+1); renderAll(); });
  reset.addEventListener("click", ()=>{ setVal(0); renderAll(); });

  c.appendChild(minus); c.appendChild(reset); c.appendChild(val); c.appendChild(plus);
}

function countKan(){
  return state.melds.filter(m=>m.type==="MINKAN"||m.type==="ANKAN").length;
}
function countFilled(arr){
  return arr.filter(x=>x && x.length>0).length;
}

function ensureKanArrays(){
  const k = countKan();
  if (state.kanDoraIndicators.length !== k) {
    const next = Array(k).fill("");
    for (let i=0;i<Math.min(k, state.kanDoraIndicators.length); i++) next[i]=state.kanDoraIndicators[i];
    state.kanDoraIndicators = next;
  }
  if (state.kanUraIndicators.length !== k) {
    const next = Array(k).fill("");
    for (let i=0;i<Math.min(k, state.kanUraIndicators.length); i++) next[i]=state.kanUraIndicators[i];
    state.kanUraIndicators = next;
  }
  if (state.riichi === "NONE") {
    state.uraIndicators = [];
    state.kanUraIndicators = Array(k).fill(""); // keep length but hidden? -> still keep for later; but easiest: clear
  } else {
    const filled = countFilled(state.doraIndicators);
    if (state.uraIndicators.length !== filled) {
      const next = Array(filled).fill("");
      for (let i=0;i<Math.min(filled, state.uraIndicators.length); i++) next[i]=state.uraIndicators[i];
      state.uraIndicators = next;
    }
    // kan ura length is k (already ensured)
  }
}

function buildSlots(containerId, kind, slots){
  const c = el(containerId);
  c.innerHTML = "";
  slots.forEach((code, idx)=>{
    const d = document.createElement("div");
    d.className = "slot " + (code ? "filled" : "empty");
    if (state.target.kind===kind && state.target.index===idx) d.classList.add("selected");

    d.addEventListener("click", ()=>{
      state.target = { kind, index: idx };
      renderAll();
    });

    if (code){
      d.innerHTML = svgTile(codeToLabel(code)) + `<div class="x">×</div>`;
      d.querySelector(".x").addEventListener("click", (e)=>{
        e.stopPropagation();
        setSlot(kind, idx, "");
        renderAll();
      });
    } else {
      d.textContent = "";
    }
    c.appendChild(d);
  });
}

function setSlot(kind, idx, code){
  if (kind==="hand") state.hand[idx]=code;
  if (kind==="win") state.win=code;
  if (kind==="dora") state.doraIndicators[idx]=code;
  if (kind==="kanDora") state.kanDoraIndicators[idx]=code;
  if (kind==="ura") state.uraIndicators[idx]=code;
  if (kind==="kanUra") state.kanUraIndicators[idx]=code;
}

function moveTargetForward(){
  const {kind,index} = state.target;
  if (kind==="hand") {
    const next = Math.min(12, index+1);
    state.target = { kind:"hand", index: next };
  }
}

function renderMelds(){
  const c = el("melds");
  c.innerHTML = "";

  for (let i=0;i<state.melds.length;i++){
    const m = state.melds[i];
    const row = document.createElement("div");
    row.className="meld";

    const left = document.createElement("div");
    left.className="left";
    for (const t of m.tiles){
      const span = document.createElement("span");
      span.innerHTML = svgTile(codeToLabel(t));
      left.appendChild(span);
    }
    const tag = document.createElement("span");
    tag.className="tag";
    tag.textContent = m.type==="CHI"?"チー":m.type==="PON"?"ポン":m.type==="MINKAN"?"明槓":"暗槓";
    left.appendChild(tag);

    const del = document.createElement("button");
    del.className="btn2";
    del.textContent="削除";
    del.addEventListener("click", ()=>{
      state.melds.splice(i,1);
      // if meldMode is kan-related, also clear draft
      state.meldDraft = [];
      ensureKanArrays();
      renderAll();
    });

    row.appendChild(left);
    row.appendChild(del);
    c.appendChild(row);
  }

  // draft preview
  if (state.meldMode !== "NORMAL"){
    const row = document.createElement("div");
    row.className="meld";
    const left = document.createElement("div");
    left.className="left";
    for (const t of state.meldDraft){
      const span = document.createElement("span");
      span.innerHTML = svgTile(codeToLabel(t));
      left.appendChild(span);
    }
    const tag = document.createElement("span");
    tag.className="tag";
    tag.textContent = "編集中";
    left.appendChild(tag);

    const del = document.createElement("button");
    del.className="btn2";
    del.textContent="戻す";
    del.addEventListener("click", ()=>{
      state.meldDraft.pop();
      renderAll();
    });

    row.appendChild(left);
    row.appendChild(del);
    c.appendChild(row);
  }
}

function meldNeedCount(){
  if (state.meldMode==="CHI" || state.meldMode==="PON") return 3;
  if (state.meldMode==="MINKAN" || state.meldMode==="ANKAN") return 4;
  return 0;
}

function validateMeld(type, tiles){
  // tiles: array of codes length 3 or 4
  const base = tiles.map(normalizeToBase); // convert red 0? to 5? but keep suit
  if (type==="CHI"){
    // must be suited sequence
    if (base.some(t=>HONORS.includes(t))) return "チーは数牌のみです";
    const suit = base[0][1];
    if (!base.every(t=>t[1]===suit)) return "チーは同一色です";
    const nums = base.map(t=>Number(t[0]==="0"?"5":t[0])).sort((a,b)=>a-b);
    if (!(nums[1]===nums[0]+1 && nums[2]===nums[0]+2)) return "チーは順子(連番)にしてください";
    return null;
  }
  if (type==="PON"){
    if (!(base[0]===base[1] && base[1]===base[2])) return "ポンは同じ牌を3枚です";
    return null;
  }
  if (type==="MINKAN" || type==="ANKAN"){
    if (!(base[0]===base[1] && base[1]===base[2] && base[2]===base[3])) return "カンは同じ牌を4枚です";
    return null;
  }
  return null;
}

function normalizeToBase(code){
  // treat red 0m as 5m for identity checks
  if (code[0]==="0") return `5${code[1]}`;
  return code;
}

// ====== Palette ======
function renderPalette(){
  const p = el("palette");
  p.innerHTML = "";
  for (const code of allPaletteCodes()){
    let displayCode = code;
    if (state.akaOn && (code==="5m"||code==="5p"||code==="5s")) {
      displayCode = "0"+code[1];
    }
    const b = document.createElement("button");
    b.className="tile";
    b.innerHTML = svgTile(codeToLabel(displayCode));
    b.addEventListener("click", ()=>onPick(displayCode));
    p.appendChild(b);
  }
}

function onPick(code){
  if (state.meldMode !== "NORMAL"){
    state.meldDraft.push(code);
    const need = meldNeedCount();
    if (state.meldDraft.length === need){
      const err = validateMeld(state.meldMode, state.meldDraft);
      if (err) {
        alert(err);
        state.meldDraft = [];
        renderAll();
        return;
      }
      state.melds.push({ type: state.meldMode, tiles: [...state.meldDraft] });
      state.meldDraft = [];
      ensureKanArrays();
    }
    renderAll();
    return;
  }

  // normal: place into selected slot
  const {kind,index} = state.target;
  setSlot(kind, index, code);

  // auto-advance only for hand input
  if (kind==="hand") moveTargetForward();

  // if riichi off, keep ura hidden; still allow dora placement
  ensureKanArrays();
  renderAll();
}

// ====== Build segments/counters ======
function renderTopControls(){
  buildSeg("roundWind", [
    {label:"東",value:"E"},{label:"南",value:"S"},{label:"西",value:"W"},{label:"北",value:"N"}
  ], ()=>state.roundWind, (v)=>state.roundWind=v);

  buildSeg("seatWind", [
    {label:"東",value:"E"},{label:"南",value:"S"},{label:"西",value:"W"},{label:"北",value:"N"}
  ], ()=>state.seatWind, (v)=>state.seatWind=v);

  buildCounter("kyotaku", ()=>state.kyotaku, (v)=>state.kyotaku=v);
  buildCounter("honba",   ()=>state.honba, (v)=>state.honba=v);

  buildSeg("winTypeSeg", [
    {label:"ロン",value:"RON"},{label:"ツモ",value:"TSUMO"}
  ], ()=>state.winType, (v)=>state.winType=v);

  buildSeg("dealerSeg", [
    {label:"親",value:true},{label:"子",value:false}
  ], ()=>state.dealer, (v)=>state.dealer=v);

  buildSeg("riichiSeg", [
    {label:"なし",value:"NONE"},{label:"立直",value:"RIICHI"},{label:"ダブル立直",value:"DOUBLE"}
  ], ()=>state.riichi, (v)=>{ state.riichi=v; ensureKanArrays(); });

  buildSeg("ippatsuSeg", [
    {label:"なし",value:false},{label:"一発",value:true}
  ], ()=>state.ippatsu, (v)=>state.ippatsu=v);

  buildSeg("rinshanSeg", [
    {label:"なし",value:false},{label:"嶺上開花",value:true}
  ], ()=>state.rinshan, (v)=>state.rinshan=v);

  buildSeg("chankanSeg", [
    {label:"なし",value:false},{label:"搶槓",value:true}
  ], ()=>state.chankan, (v)=>state.chankan=v);

  buildSeg("haiteiSeg", [
    {label:"なし",value:"NONE"},{label:"海底/河底",value:"BOTH"}
  ], ()=>state.haitei, (v)=>state.haitei=v);

  buildSeg("tenhouSeg", [
    {label:"なし",value:"NONE"},{label:"天和",value:"TENHOU"},{label:"地和",value:"CHIHOU"}
  ], ()=>state.tenhou, (v)=>state.tenhou=v);
}

// ====== Buttons ======
function bindButtons(){
  el("modeChi").addEventListener("click", ()=>{ state.meldMode="CHI"; state.meldDraft=[]; renderAll(); });
  el("modePon").addEventListener("click", ()=>{ state.meldMode="PON"; state.meldDraft=[]; renderAll(); });
  el("modeMinkan").addEventListener("click", ()=>{ state.meldMode="MINKAN"; state.meldDraft=[]; renderAll(); });
  el("modeAnkan").addEventListener("click", ()=>{ state.meldMode="ANKAN"; state.meldDraft=[]; renderAll(); });
  el("modeNormal").addEventListener("click", ()=>{ state.meldMode="NORMAL"; state.meldDraft=[]; renderAll(); });

  el("toggleAka").addEventListener("click", ()=>{
    state.akaOn = !state.akaOn;
    renderAll();
  });

  el("fillSample").addEventListener("click", ()=>{
    // sample: menzen riichi ron pinfu-like, plus one kan to show kan-dora slots
    state.roundWind="E";
    state.seatWind="S";
    state.kyotaku=0;
    state.honba=1;
    state.winType="RON";
    state.dealer=true;
    state.riichi="RIICHI";
    state.ippatsu=false;
    state.rinshan=false;
    state.chankan=false;
    state.haitei="NONE";
    state.tenhou="NONE";
    state.akaOn=false;

    state.hand = ["1m","2m","3m","3p","4p","5p","6s","7s","8s","2m","3m","4m","5p"];
    state.win = "5p";

    state.melds = [{ type:"ANKAN", tiles:["5s","5s","5s","5s"] }]; // kan -> kan dora slot appears
    state.meldMode="NORMAL";
    state.meldDraft=[];

    state.doraIndicators = ["4s","","","",""];
    ensureKanArrays();
    state.kanDoraIndicators = [""];
    // riichi on => ura length = filled dora (1)
    state.uraIndicators = [""];
    state.kanUraIndicators = [""];

    // set default target
    state.target = { kind:"hand", index:0 };
    renderAll();
  });

  el("clearAll").addEventListener("click", ()=>{
    state.target = { kind:"hand", index:0 };
    state.kyotaku=0; state.honba=0;
    state.hand = Array(13).fill("");
    state.win = "";
    state.doraIndicators = Array(5).fill("");
    state.melds = [];
    state.meldDraft=[];
    state.meldMode="NORMAL";
    state.riichi="NONE";
    state.ippatsu=false;
    state.rinshan=false;
    state.chankan=false;
    state.haitei="NONE";
    state.tenhou="NONE";
    ensureKanArrays();
    renderAll();
  });

  el("calc").addEventListener("click", async ()=>{
    // basic checks
    if (state.hand.some(x=>!x)) return alert("手牌13枚をすべて埋めてください");
    if (!state.win) return alert("和了牌を入れてください");

    // riichi constraints: ippatsu only with riichi
    if (state.ippatsu && state.riichi==="NONE") return alert("一発は立直時のみです");

    // Build request JSON (structured, safe)
    const req = buildRequestJson();
    el("jsonOut").textContent = JSON.stringify(req, null, 2);

    const res = await fetch("/api/score", {
      method:"POST",
      headers:{ "content-type":"application/json" },
      body: JSON.stringify(req),
    });
    const text = await res.text();
    try {
      el("result").textContent = JSON.stringify(JSON.parse(text), null, 2);
    } catch {
      el("result").textContent = text;
    }
  });
}

function buildRequestJson(){
  // dora indicator lists: keep only filled (and keep order)
  const dora_ind = state.doraIndicators.filter(Boolean);
  const kan_dora_ind = state.kanDoraIndicators.filter(Boolean);
  const ura_ind = (state.riichi==="NONE") ? [] : state.uraIndicators.filter(Boolean);
  const kan_ura_ind = (state.riichi==="NONE") ? [] : state.kanUraIndicators.filter(Boolean);

  // map winds
  const windLabel = (w)=> windCodeToLabel(w);
  const round_seat = `${windLabel(state.roundWind)}場の${windLabel(state.seatWind)}家`;

  // for now, to keep your existing Rust parsing easier, also provide readable strings if you want:
  // (you can remove these later)
  const readableTiles = (codes)=>codes.map(codeToLabel).join(" ");

  // haitei/houtei mapping: your earlier CLI separated; here unify
  const haitei = state.haitei==="BOTH" && state.winType==="TSUMO";
  const houtei = state.haitei==="BOTH" && state.winType==="RON";

  const tenhou = state.tenhou==="TENHOU";
  const chihou = state.tenhou==="CHIHOU";

  return {
    round_wind: state.roundWind,
    seat_wind: state.seatWind,
    kyotaku: state.kyotaku,
    honba: state.honba,
    win_type: state.winType,
    dealer: state.dealer,

    hand_tiles: [...state.hand],
    win_tile: state.win,
    melds: state.melds.map(m=>({ type:m.type, tiles:[...m.tiles] })),

    dora_indicators: dora_ind,
    kan_dora_indicators: kan_dora_ind,
    ura_indicators: ura_ind,
    kan_ura_indicators: kan_ura_ind,

    flags: {
      riichi: state.riichi, // NONE/RIICHI/DOUBLE
      ippatsu: state.ippatsu,
      rinshan: state.rinshan,
      chankan: state.chankan,
      haitei,
      houtei,
      tenhou,
      chihou,
    },

    // (optional) readable strings for debugging / compatibility
    _readable: {
      hand: readableTiles(state.hand),
      win_tile: codeToLabel(state.win),
      dora_indicators: readableTiles(dora_ind) || "なし",
      ura_indicators: readableTiles(ura_ind) || "なし",
      kan_dora_indicators: readableTiles(kan_dora_ind) || "なし",
      kan_ura_indicators: readableTiles(kan_ura_ind) || "なし",
      round_seat,
      melds: state.melds.map(m=>`${m.tiles.map(codeToLabel).join("")}(${m.type})`).join(" "),
    }
  };
}

function windCodeToLabel(code){
  return code==="E"?"東":code==="S"?"南":code==="W"?"西":"北";
}

// ====== Rendering ======
function renderSlotsAll(){
  ensureKanArrays();

  buildSlots("handSlots", "hand", state.hand);
  buildSlots("winSlot", "win", [state.win]);

  buildSlots("doraSlots", "dora", state.doraIndicators);
  buildSlots("kanDoraSlots", "kanDora", state.kanDoraIndicators.length ? state.kanDoraIndicators : Array(0).fill(""));

  // ura slots visible only when riichi != NONE
  const uraVisible = state.riichi !== "NONE";
  el("uraSlots").style.opacity = uraVisible ? "1" : "0.35";
  el("kanUraSlots").style.opacity = uraVisible ? "1" : "0.35";

  const uraSlots = uraVisible ? state.uraIndicators : [];
  const kanUraSlots = uraVisible ? state.kanUraIndicators : [];

  buildSlots("uraSlots", "ura", uraSlots.length ? uraSlots : Array(0).fill(""));
  buildSlots("kanUraSlots", "kanUra", kanUraSlots.length ? kanUraSlots : Array(0).fill(""));
}

function renderMeldControls(){
  // mode buttons
  const setActive = (id, on)=> el(id).classList.toggle("active", on);
  setActive("modeChi", state.meldMode==="CHI");
  setActive("modePon", state.meldMode==="PON");
  setActive("modeMinkan", state.meldMode==="MINKAN");
  setActive("modeAnkan", state.meldMode==="ANKAN");
  setActive("modeNormal", state.meldMode==="NORMAL");

  el("toggleAka").classList.toggle("active", state.akaOn);
  el("toggleAka").textContent = `赤牌切替：${state.akaOn ? "ON" : "OFF"}`;

  const hint = el("meldHint");
  if (state.meldMode==="NORMAL"){
    hint.textContent = "通常入力：選択した枠に牌を入れます（手牌は自動で次へ）。";
  } else {
    const need = meldNeedCount();
    const modeName = state.meldMode==="CHI"?"チー":state.meldMode==="PON"?"ポン":state.meldMode==="MINKAN"?"明槓":"暗槓";
    hint.textContent = `${modeName}編集中：牌パレットから ${need}枚 選ぶと副露が確定します（順子/同牌チェックあり）。`;
  }
}

function renderJsonPreview(){
  const req = buildRequestJson();
  el("jsonOut").textContent = JSON.stringify(req, null, 2);
}

function renderAll(){
  ensureKanArrays();
  renderTopControls();
  renderSlotsAll();
  renderMeldControls();
  renderMelds();
  renderPalette();
  renderJsonPreview();
}

// ====== Init ======
function init(){
  bindButtons();
  renderAll();

  // initial target
  state.target = { kind:"hand", index:0 };

  // default: select first hand slot
  renderAll();
}
init();
