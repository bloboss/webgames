// ============================================================
// Water Sort Puzzle - Thin JS Frontend
// ============================================================

let wasm = null;
let game = null;
let difficulty = "easy";
let selectedBottle = -1;
let timerInterval = null;
let elapsedSeconds = 0;
let gameActive = false;
let hintSrc = -1;
let hintDst = -1;

// ---- Cookie/LocalStorage ----

function loadData(key) {
    try {
        const raw = localStorage.getItem(key);
        return raw ? JSON.parse(raw) : null;
    } catch (e) {
        return null;
    }
}

function saveData(key, data) {
    try {
        localStorage.setItem(key, JSON.stringify(data));
    } catch (e) {}
}

function loadStats() {
    return loadData("watersort_stats") || {
        easy:   { played: 0, won: 0, bestTime: null, bestMoves: null, totalTime: 0 },
        medium: { played: 0, won: 0, bestTime: null, bestMoves: null, totalTime: 0 },
        hard:   { played: 0, won: 0, bestTime: null, bestMoves: null, totalTime: 0 },
    };
}

function saveStats(stats) {
    saveData("watersort_stats", stats);
}

function loadHistory() {
    return loadData("watersort_history") || [];
}

function saveHistory(history) {
    if (history.length > 50) history = history.slice(-50);
    saveData("watersort_history", history);
}

function nowString() {
    const d = new Date();
    return d.toISOString().slice(0, 19).replace("T", " ");
}

function recordWin(diff, time, moves) {
    const stats = loadStats();
    const s = stats[diff];
    s.played++;
    s.won++;
    s.totalTime += time;
    if (s.bestTime === null || time < s.bestTime) s.bestTime = time;
    if (s.bestMoves === null || moves < s.bestMoves) s.bestMoves = moves;
    saveStats(stats);

    const history = loadHistory();
    history.push({ date: nowString(), difficulty: diff, result: "Won", time, moves });
    saveHistory(history);
}

function recordAbandon(diff, time, moves) {
    const stats = loadStats();
    stats[diff].played++;
    stats[diff].totalTime += time;
    saveStats(stats);

    const history = loadHistory();
    history.push({ date: nowString(), difficulty: diff, result: "Abandoned", time, moves });
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
    updateTimerDisplay();
    timerInterval = setInterval(() => {
        elapsedSeconds++;
        updateTimerDisplay();
    }, 1000);
}

function stopTimer() {
    if (timerInterval) {
        clearInterval(timerInterval);
        timerInterval = null;
    }
    gameActive = false;
}

function updateTimerDisplay() {
    document.getElementById("timer-display").textContent = "Time: " + formatTime(elapsedSeconds);
}

function updateMoveCounter() {
    if (!game) return;
    document.getElementById("move-counter").textContent = "Moves: " + game.move_count();
}

// ---- Rendering ----

function renderBottles() {
    if (!game) return;
    const container = document.getElementById("bottles-container");
    const bottlesData = JSON.parse(game.get_bottles_json());

    container.innerHTML = "";

    bottlesData.forEach((units, idx) => {
        const bottle = document.createElement("div");
        bottle.className = "bottle";
        if (idx === selectedBottle) bottle.classList.add("selected");
        if (idx === hintSrc) bottle.classList.add("hint-src");
        if (idx === hintDst) bottle.classList.add("hint-dst");

        bottle.addEventListener("click", () => onBottleClick(idx));

        // Render units bottom to top
        units.forEach((color) => {
            const unit = document.createElement("div");
            unit.className = "unit color-" + color;
            bottle.appendChild(unit);
        });

        container.appendChild(bottle);
    });

    updateMoveCounter();
}

// ---- Game Logic ----

function newGame() {
    if (game && gameActive) {
        recordAbandon(difficulty, elapsedSeconds, game.move_count());
    }

    game = new wasm.WaterSortGame(difficulty);
    selectedBottle = -1;
    hintSrc = -1;
    hintDst = -1;
    setMessage("");
    renderBottles();
    startTimer();
}

function resetGame() {
    if (!game) return;
    game.reset();
    selectedBottle = -1;
    hintSrc = -1;
    hintDst = -1;
    setMessage("Board reset.");
    renderBottles();
    startTimer();
}

function undoMove() {
    if (!game || !gameActive) return;
    if (game.undo()) {
        selectedBottle = -1;
        hintSrc = -1;
        hintDst = -1;
        setMessage("");
        renderBottles();
    } else {
        setMessage("Nothing to undo.");
    }
}

function getHint() {
    if (!game || !gameActive) return;
    const hint = game.get_hint();
    if (hint === "null") {
        setMessage("No solution found from current state.");
        return;
    }
    const [src, dst] = JSON.parse(hint);
    hintSrc = src;
    hintDst = dst;
    selectedBottle = -1;
    setMessage("Hint: pour bottle " + (src + 1) + " into bottle " + (dst + 1));
    renderBottles();

    // Clear hint highlight after 2 seconds
    setTimeout(() => {
        if (hintSrc === src && hintDst === dst) {
            hintSrc = -1;
            hintDst = -1;
            renderBottles();
        }
    }, 2000);
}

function onBottleClick(idx) {
    if (!game || !gameActive) return;
    hintSrc = -1;
    hintDst = -1;

    if (selectedBottle === -1) {
        // Select source bottle (only if not empty)
        const bottles = JSON.parse(game.get_bottles_json());
        if (bottles[idx].length > 0) {
            selectedBottle = idx;
            renderBottles();
        }
    } else if (selectedBottle === idx) {
        // Deselect
        selectedBottle = -1;
        renderBottles();
    } else {
        // Try to pour
        const src = selectedBottle;
        const dst = idx;
        selectedBottle = -1;

        const poured = game.pour(src, dst);
        if (poured > 0) {
            setMessage("");
            renderBottles();

            if (game.is_solved()) {
                stopTimer();
                recordWin(difficulty, elapsedSeconds, game.move_count());
                setMessage("Puzzle solved in " + game.move_count() + " moves! Time: " + formatTime(elapsedSeconds), true);
            }
        } else {
            setMessage("Invalid pour.");
            renderBottles();
        }
    }
}

function setMessage(msg, isWin) {
    const el = document.getElementById("message");
    el.textContent = msg;
    el.className = isWin ? "win" : "";
}

// ---- Stats & History ----

function renderStatsTable() {
    const stats = loadStats();
    let html = "<table><thead><tr><th>Diff</th><th>Played</th><th>Won</th><th>Best Time</th><th>Best Moves</th><th>Avg Time</th></tr></thead><tbody>";
    for (const d of ["easy", "medium", "hard"]) {
        const s = stats[d];
        const best = s.bestTime !== null ? formatTime(s.bestTime) : "--:--";
        const bestM = s.bestMoves !== null ? s.bestMoves : "--";
        const avg = s.won > 0 ? formatTime(Math.floor(s.totalTime / s.won)) : "--:--";
        const label = d.charAt(0).toUpperCase() + d.slice(1);
        html += "<tr><td>" + label + "</td><td>" + s.played + "</td><td>" + s.won +
                "</td><td>" + best + "</td><td>" + bestM + "</td><td>" + avg + "</td></tr>";
    }
    html += "</tbody></table>";
    return html;
}

function renderHistoryTable() {
    const history = loadHistory();
    if (history.length === 0) return "<p>No games played yet.</p>";
    let html = "<table><thead><tr><th>Date</th><th>Diff</th><th>Result</th><th>Time</th><th>Moves</th></tr></thead><tbody>";
    const recent = history.slice(-20).reverse();
    for (const h of recent) {
        const cls = h.result === "Won" ? "result-won" : "result-abandoned";
        html += "<tr><td>" + h.date + "</td><td>" + h.difficulty +
                "</td><td class=\"" + cls + "\">" + h.result +
                "</td><td>" + formatTime(h.time) + "</td><td>" + h.moves + "</td></tr>";
    }
    html += "</tbody></table>";
    return html;
}

// ---- Event Setup ----

function setupEvents() {
    // Difficulty buttons
    document.querySelectorAll(".diff-btn").forEach(btn => {
        btn.addEventListener("click", () => {
            document.querySelectorAll(".diff-btn").forEach(b => b.classList.remove("active"));
            btn.classList.add("active");
            difficulty = btn.dataset.diff;
        });
    });

    document.getElementById("btn-new").addEventListener("click", newGame);
    document.getElementById("btn-reset").addEventListener("click", resetGame);
    document.getElementById("btn-undo").addEventListener("click", undoMove);
    document.getElementById("btn-hint").addEventListener("click", getHint);

    // Stats toggle
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

    // Keyboard: Escape to deselect
    document.addEventListener("keydown", (e) => {
        if (e.key === "Escape") {
            selectedBottle = -1;
            hintSrc = -1;
            hintDst = -1;
            renderBottles();
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
        wasm = await import("../pkg/water_sort.js");
        await wasm.default();
    } catch (e) {
        try {
            wasm = await import("./pkg/water_sort.js");
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
