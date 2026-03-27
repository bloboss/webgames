// ============================================================
// Number Match Puzzle - JS Frontend
// ============================================================

let wasm = null;
let game = null;
let initialRows = 3;
let selectedCell = -1;
let timerInterval = null;
let elapsedSeconds = 0;
let gameActive = false;
let hintA = -1;
let hintB = -1;

// ---- Storage ----

function loadData(key) {
    try {
        const raw = localStorage.getItem(key);
        return raw ? JSON.parse(raw) : null;
    } catch (e) { return null; }
}

function saveData(key, data) {
    try { localStorage.setItem(key, JSON.stringify(data)); } catch (e) {}
}

function loadStats() {
    return loadData("numbermatch_stats") || {
        "3": { played: 0, won: 0, bestScore: null, bestTime: null, totalTime: 0 },
        "5": { played: 0, won: 0, bestScore: null, bestTime: null, totalTime: 0 },
        "7": { played: 0, won: 0, bestScore: null, bestTime: null, totalTime: 0 },
    };
}

function saveStats(stats) { saveData("numbermatch_stats", stats); }

function loadHistory() { return loadData("numbermatch_history") || []; }

function saveHistory(history) {
    if (history.length > 50) history = history.slice(-50);
    saveData("numbermatch_history", history);
}

function nowString() {
    return new Date().toISOString().slice(0, 19).replace("T", " ");
}

function recordWin(rows, time, score, matches) {
    const stats = loadStats();
    const key = String(rows);
    if (!stats[key]) stats[key] = { played: 0, won: 0, bestScore: null, bestTime: null, totalTime: 0 };
    const s = stats[key];
    s.played++;
    s.won++;
    s.totalTime += time;
    if (s.bestScore === null || score > s.bestScore) s.bestScore = score;
    if (s.bestTime === null || time < s.bestTime) s.bestTime = time;
    saveStats(stats);

    const history = loadHistory();
    history.push({ date: nowString(), rows, result: "Won", time, score, matches });
    saveHistory(history);
}

function recordLoss(rows, time, score, matches) {
    const stats = loadStats();
    const key = String(rows);
    if (!stats[key]) stats[key] = { played: 0, won: 0, bestScore: null, bestTime: null, totalTime: 0 };
    stats[key].played++;
    stats[key].totalTime += time;
    saveStats(stats);

    const history = loadHistory();
    history.push({ date: nowString(), rows, result: "Abandoned", time, score, matches });
    saveHistory(history);
}

// ---- Timer ----

function formatTime(seconds) {
    const m = Math.floor(seconds / 60);
    const s = seconds % 60;
    return String(m).padStart(2, "0") + ":" + String(s).padStart(2, "0");
}

function startTimer() {
    stopTimer();
    elapsedSeconds = 0;
    gameActive = true;
    updateTimer();
    timerInterval = setInterval(() => { elapsedSeconds++; updateTimer(); }, 1000);
}

function stopTimer() {
    if (timerInterval) { clearInterval(timerInterval); timerInterval = null; }
    gameActive = false;
}

function updateTimer() {
    document.getElementById("timer-display").textContent = "Time: " + formatTime(elapsedSeconds);
}

function updateInfoBar() {
    if (!game) return;
    document.getElementById("score-display").textContent = "Score: " + game.score();
    document.getElementById("matches-display").textContent = "Matches: " + game.matches_made();
    document.getElementById("remaining-display").textContent = "Left: " + game.remaining();
}

// ---- Rendering ----

function renderGrid() {
    if (!game) return;
    const container = document.getElementById("grid-container");
    const gridData = JSON.parse(game.get_grid_json());
    const cols = game.cols();

    container.style.gridTemplateColumns = "repeat(" + cols + ", 44px)";
    container.innerHTML = "";

    gridData.forEach((val, idx) => {
        const cell = document.createElement("div");
        cell.className = "cell";

        if (val === 0) {
            cell.classList.add("empty");
        } else {
            cell.textContent = val;
            cell.classList.add("d" + val);
            if (idx === selectedCell) cell.classList.add("selected");
            if (idx === hintA || idx === hintB) cell.classList.add("hint");
            cell.addEventListener("click", () => onCellClick(idx));
        }

        container.appendChild(cell);
    });

    updateInfoBar();
    checkNoMoves();
}

function checkNoMoves() {
    const banner = document.getElementById("no-moves-banner");
    if (game && !game.is_over() && !game.has_moves()) {
        banner.classList.remove("hidden");
    } else {
        banner.classList.add("hidden");
    }
}

// ---- Game Logic ----

function newGame() {
    if (game && gameActive) {
        recordLoss(initialRows, elapsedSeconds, game.score(), game.matches_made());
    }
    game = new wasm.NumberMatchGame(initialRows);
    selectedCell = -1;
    hintA = -1;
    hintB = -1;
    setMessage("");
    renderGrid();
    startTimer();
}

function onCellClick(idx) {
    if (!game || !gameActive) return;
    hintA = -1;
    hintB = -1;

    const val = game.cell_value(idx);
    if (val === 0) return;

    if (selectedCell === -1) {
        selectedCell = idx;
        renderGrid();
    } else if (selectedCell === idx) {
        selectedCell = -1;
        renderGrid();
    } else {
        const a = selectedCell;
        selectedCell = -1;

        if (game.make_match(a, idx)) {
            setMessage("+10");
            renderGrid();

            // Brief flash on matched cells
            flashCells([a, idx], "matched");

            if (game.is_won()) {
                stopTimer();
                recordWin(initialRows, elapsedSeconds, game.score(), game.matches_made());
                setMessage("Board cleared! Score: " + game.score() + " Time: " + formatTime(elapsedSeconds), true);
            }
        } else {
            setMessage("Invalid match.");
            renderGrid();
            flashCells([a, idx], "invalid-flash");
        }
    }
}

function flashCells(indices, className) {
    const container = document.getElementById("grid-container");
    const cells = container.children;
    indices.forEach(idx => {
        if (idx < cells.length) {
            cells[idx].classList.add(className);
            setTimeout(() => cells[idx].classList.remove(className), 300);
        }
    });
}

function addRow() {
    if (!game || !gameActive) return;
    if (game.add_row()) {
        setMessage("Row added. -20 points.");
        renderGrid();
    } else {
        setMessage("Cannot add more rows.");
    }
}

function undoMove() {
    if (!game || !gameActive) return;
    if (game.undo()) {
        selectedCell = -1;
        hintA = -1;
        hintB = -1;
        setMessage("");
        renderGrid();
    } else {
        setMessage("Nothing to undo.");
    }
}

function getHint() {
    if (!game || !gameActive) return;
    const hint = game.get_hint();
    if (hint === "null") {
        setMessage("No matches available. Try adding a row.");
        return;
    }
    const [a, b] = JSON.parse(hint);
    hintA = a;
    hintB = b;
    selectedCell = -1;
    setMessage("Hint: cells highlighted.");
    renderGrid();

    setTimeout(() => {
        if (hintA === a && hintB === b) {
            hintA = -1;
            hintB = -1;
            renderGrid();
        }
    }, 2500);
}

function setMessage(msg, isWin) {
    const el = document.getElementById("message");
    el.textContent = msg;
    el.className = isWin ? "win" : "";
}

// ---- Stats & History ----

function renderStatsTable() {
    const stats = loadStats();
    let html = "<table><thead><tr><th>Size</th><th>Played</th><th>Won</th><th>Best Score</th><th>Best Time</th></tr></thead><tbody>";
    for (const r of ["3", "5", "7"]) {
        const s = stats[r] || { played: 0, won: 0, bestScore: null, bestTime: null };
        const bs = s.bestScore !== null ? s.bestScore : "--";
        const bt = s.bestTime !== null ? formatTime(s.bestTime) : "--:--";
        html += "<tr><td>" + r + " rows</td><td>" + s.played + "</td><td>" + s.won +
                "</td><td>" + bs + "</td><td>" + bt + "</td></tr>";
    }
    html += "</tbody></table>";
    return html;
}

function renderHistoryTable() {
    const history = loadHistory();
    if (history.length === 0) return "<p>No games played yet.</p>";
    let html = "<table><thead><tr><th>Date</th><th>Size</th><th>Result</th><th>Score</th><th>Matches</th><th>Time</th></tr></thead><tbody>";
    const recent = history.slice(-20).reverse();
    for (const h of recent) {
        const cls = h.result === "Won" ? "result-won" : "result-lost";
        html += "<tr><td>" + h.date + "</td><td>" + h.rows + "r</td><td class=\"" + cls + "\">" +
                h.result + "</td><td>" + h.score + "</td><td>" + h.matches +
                "</td><td>" + formatTime(h.time) + "</td></tr>";
    }
    html += "</tbody></table>";
    return html;
}

// ---- Events ----

function setupEvents() {
    document.querySelectorAll(".size-btn").forEach(btn => {
        btn.addEventListener("click", () => {
            document.querySelectorAll(".size-btn").forEach(b => b.classList.remove("active"));
            btn.classList.add("active");
            initialRows = parseInt(btn.dataset.rows);
        });
    });

    document.getElementById("btn-new").addEventListener("click", newGame);
    document.getElementById("btn-undo").addEventListener("click", undoMove);
    document.getElementById("btn-hint").addEventListener("click", getHint);
    document.getElementById("btn-add-row").addEventListener("click", addRow);
    document.getElementById("btn-banner-add").addEventListener("click", addRow);

    document.getElementById("btn-toggle-stats").addEventListener("click", () => {
        const el = document.getElementById("stats-display");
        if (el.classList.contains("hidden")) {
            el.innerHTML = renderStatsTable();
            el.classList.remove("hidden");
            document.getElementById("btn-toggle-stats").textContent = "Hide Stats";
        } else {
            el.classList.add("hidden");
            document.getElementById("btn-toggle-stats").textContent = "Show Stats";
        }
    });

    document.getElementById("btn-toggle-history").addEventListener("click", () => {
        const el = document.getElementById("history-display");
        if (el.classList.contains("hidden")) {
            el.innerHTML = renderHistoryTable();
            el.classList.remove("hidden");
            document.getElementById("btn-toggle-history").textContent = "Hide History";
        } else {
            el.classList.add("hidden");
            document.getElementById("btn-toggle-history").textContent = "Show History";
        }
    });

    document.addEventListener("keydown", (e) => {
        if (e.key === "Escape") {
            selectedCell = -1;
            hintA = -1;
            hintB = -1;
            renderGrid();
        }
        if (e.key === "z" && (e.ctrlKey || e.metaKey)) {
            e.preventDefault();
            undoMove();
        }
    });
}

// ---- Init ----

async function init() {
    try {
        wasm = await import("../pkg/number_match.js");
        await wasm.default();
    } catch (e) {
        try {
            wasm = await import("./pkg/number_match.js");
            await wasm.default();
        } catch (e2) {
            document.getElementById("message").textContent =
                "Error loading WASM. Run: wasm-pack build --target web";
            console.error("WASM load error:", e, e2);
            return;
        }
    }

    setupEvents();
    newGame();
}

init();
