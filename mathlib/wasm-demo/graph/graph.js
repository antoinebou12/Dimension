/**
 * mathlib WASM demo — Graph (pathfinding, coloring, BFS/DFS).
 */
import {
  initLib, byId, showError, needBuild, needRebuild,
  drawGraphOnCanvas, drawGraphColoringOnCanvas, drawGraphTreeOnCanvas,
} from "../shared.js";

try {
  const lib = await initLib();
  const { WasmGraph, WasmMatrix } = lib;

  function buildUndirectedGraph(n, edges) {
    const g = new WasmGraph(n);
    for (let i = 0; i < edges.length; i += 3) {
      g.addEdgeUndirected(edges[i], edges[i + 1], edges[i + 2]);
    }
    return g;
  }

  function graphCoordsForLayout(n) {
    const out = [];
    for (let i = 0; i < n; i++) {
      const angle = (2 * Math.PI * i) / n - Math.PI / 2;
      out.push(Math.cos(angle), Math.sin(angle));
    }
    return out;
  }

  function edgesToText(edges) {
    const parts = [];
    for (let i = 0; i < edges.length; i += 3)
      parts.push(edges[i] + "→" + edges[i + 1] + "(" + edges[i + 2] + ")");
    return parts.join(", ");
  }

  function getPathAndDists(ex, algoId) {
    const gDists = WasmGraph.fromEdges(ex.n, ex.edges);
    const dres = gDists.dijkstra(ex.source);
    const dists = dres.getDistances();
    let path = [];
    let distVal = 0;
    if (algoId === "dijkstra") {
      path = dres.pathTo(ex.target);
      distVal = dists[ex.target] ?? Infinity;
    } else {
      const gPath = WasmGraph.fromEdges(ex.n, ex.edges);
      if (algoId === "astar") {
        const ares = gPath.astar(ex.source, ex.target);
        path = ares.getPath();
        distVal = ares.getDist();
      } else if (algoId === "astarCoords") {
        const coordsData = graphCoordsForLayout(ex.n);
        const coords = WasmMatrix.fromArray(ex.n, 2, coordsData);
        const ares = gPath.astarWithCoords(ex.source, ex.target, coords);
        path = ares.getPath();
        distVal = ares.getDist();
      } else if (algoId === "dstar") {
        const dres2 = gPath.dstarLite(ex.source, ex.target);
        path = dres2.getPath();
        distVal = dres2.getDist();
      }
    }
    return { path, dists, distVal };
  }

  if (typeof WasmGraph !== "function" || typeof WasmGraph.fromEdges !== "function") {
    byId("out-graph").textContent = needRebuild;
  } else {
    const GRAPH_EXAMPLES = [
      { title: "4-node", n: 4, edges: [0, 1, 1, 0, 2, 4, 1, 2, 2, 1, 3, 6, 2, 3, 1], source: 0, target: 3 },
      { title: "5-node", n: 5, edges: [0, 1, 2, 0, 2, 5, 1, 2, 1, 1, 3, 3, 2, 3, 1, 2, 4, 2, 3, 4, 4], source: 0, target: 4 },
      { title: "4-node (0→2)", n: 4, edges: [0, 1, 1, 0, 2, 4, 1, 2, 2, 1, 3, 6, 2, 3, 1], source: 0, target: 2 },
      { title: "6-node", n: 6, edges: [0, 1, 2, 0, 2, 1, 1, 3, 1, 1, 4, 3, 2, 1, 1, 2, 4, 2, 3, 5, 2, 4, 5, 1], source: 0, target: 5 },
      { title: "Chain 5", n: 5, edges: [0, 1, 1, 1, 2, 1, 2, 3, 1, 3, 4, 1], source: 0, target: 4 },
      { title: "Star", n: 6, edges: [0, 1, 1, 0, 2, 1, 0, 3, 1, 0, 4, 1, 0, 5, 1], source: 0, target: 5 },
      { title: "Grid 6", n: 6, edges: [0, 1, 1, 0, 2, 1, 1, 3, 1, 2, 3, 1, 2, 4, 1, 3, 5, 1, 4, 5, 1], source: 0, target: 5 },
      { title: "8-node", n: 8, edges: [0, 1, 1, 0, 2, 2, 1, 3, 1, 1, 4, 2, 2, 3, 1, 2, 5, 1, 3, 6, 1, 4, 6, 1, 5, 6, 1, 6, 7, 1], source: 0, target: 7 },
    ];
    const GRAPH_ALGOS = [
      { id: "dijkstra", title: "Dijkstra" },
      { id: "astar", title: "A* (zero h)" },
      { id: "astarCoords", title: "A* (Euclidean)" },
      { id: "dstar", title: "D* Lite" },
    ];
    let graphExampleIndex = 0;
    let graphAlgoIndex = 0;
    function showGraphExample(algoIndex) {
      graphAlgoIndex = algoIndex ?? graphAlgoIndex;
      const ex = GRAPH_EXAMPLES[graphExampleIndex];
      const algoId = GRAPH_ALGOS[graphAlgoIndex].id;
      const algoTitle = GRAPH_ALGOS[graphAlgoIndex].title;
      const { path, dists, distVal } = getPathAndDists(ex, algoId);
      const canvas = byId("canvas-graph");
      const ctx = canvas.getContext("2d");
      drawGraphOnCanvas(ctx, canvas.width, canvas.height, ex.n, ex.edges, path, dists, ex.source);
      byId("out-graph").textContent =
        "Graph: " + ex.n + " nodes, edges " + edgesToText(ex.edges) + "\n" +
        algoTitle + " from " + ex.source + " to " + ex.target + ":\ndistances (Dijkstra): [" +
        dists.map((x) => (x === 1 / 0 || x === Infinity ? "∞" : x.toFixed(2))).join(", ") +
        "]\npath: [" + path.join(", ") + "]\ndist: " +
        (Number.isFinite(distVal) ? distVal.toFixed(2) : "∞");
      byId("graph-algorithms").querySelectorAll("button").forEach((b, i) => b.classList.toggle("active", i === graphAlgoIndex));
    }
    function setGraphExample(index) {
      graphExampleIndex = index;
      byId("graph-examples").querySelectorAll("button").forEach((b, i) => b.classList.toggle("active", i === index));
      showGraphExample();
    }
    GRAPH_EXAMPLES.forEach((_, i) => {
      const btn = document.createElement("button");
      btn.type = "button";
      btn.textContent = GRAPH_EXAMPLES[i].title;
      btn.addEventListener("click", () => setGraphExample(i));
      byId("graph-examples").appendChild(btn);
    });
    GRAPH_ALGOS.forEach((_, i) => {
      const btn = document.createElement("button");
      btn.type = "button";
      btn.textContent = GRAPH_ALGOS[i].title;
      btn.addEventListener("click", () => showGraphExample(i));
      byId("graph-algorithms").appendChild(btn);
    });
    setGraphExample(0);
  }

  if (typeof WasmGraph !== "function" || typeof WasmGraph.prototype.greedyVertexColoring !== "function") {
    byId("out-coloring").textContent = needRebuild;
  } else {
    const COLORING_EXAMPLES = [
      { title: "4-node", n: 4, edges: [0, 1, 1, 0, 2, 1, 1, 2, 1, 1, 3, 1, 2, 3, 1] },
      { title: "Triangle", n: 3, edges: [0, 1, 1, 1, 2, 1, 2, 0, 1] },
      { title: "Path 4", n: 4, edges: [0, 1, 1, 1, 2, 1, 2, 3, 1] },
      { title: "Star 5", n: 5, edges: [0, 1, 1, 0, 2, 1, 0, 3, 1, 0, 4, 1] },
      { title: "Bipartite", n: 4, edges: [0, 2, 1, 0, 3, 1, 1, 2, 1, 1, 3, 1] },
    ];
    const COLORING_ALGOS = [
      { id: "greedy", title: "Greedy" },
      { id: "dsatur", title: "DSatur" },
      { id: "bipartite", title: "Bipartite" },
    ];
    let coloringExampleIndex = 0;
    let coloringAlgoIndex = 0;
    function showColoringExample(algoIndex) {
      coloringAlgoIndex = algoIndex ?? coloringAlgoIndex;
      const ex = COLORING_EXAMPLES[coloringExampleIndex];
      const g = buildUndirectedGraph(ex.n, ex.edges);
      const algoId = COLORING_ALGOS[coloringAlgoIndex].id;
      let colors = [];
      let text = "Graph: " + ex.n + " nodes\n";
      if (algoId === "greedy") {
        colors = g.greedyVertexColoring();
        text += "Greedy: " + (colors.length ? Math.max(...colors) + 1 : 0) + " colors\ncolors: [" + colors.join(", ") + "]";
      } else if (algoId === "dsatur") {
        colors = g.dsaturColoring();
        text += "DSatur: " + (colors.length ? Math.max(...colors) + 1 : 0) + " colors\ncolors: [" + colors.join(", ") + "]";
      } else {
        const bip = g.isBipartite();
        if (bip != null) {
          colors = bip;
          text += "Bipartite: yes (2-coloring)\ncolors: [" + colors.join(", ") + "]";
        } else {
          text += "Bipartite: no (odd cycle)";
        }
      }
      drawGraphColoringOnCanvas(
        byId("canvas-coloring").getContext("2d"),
        byId("canvas-coloring").width,
        byId("canvas-coloring").height,
        ex.n,
        ex.edges,
        colors.length ? colors : null
      );
      byId("out-coloring").textContent = text;
      byId("coloring-algos").querySelectorAll("button").forEach((b, i) => b.classList.toggle("active", i === coloringAlgoIndex));
    }
    function setColoringExample(index) {
      coloringExampleIndex = index;
      byId("coloring-examples").querySelectorAll("button").forEach((b, i) => b.classList.toggle("active", i === index));
      showColoringExample();
    }
    COLORING_EXAMPLES.forEach((_, i) => {
      const btn = document.createElement("button");
      btn.type = "button";
      btn.textContent = COLORING_EXAMPLES[i].title;
      btn.addEventListener("click", () => setColoringExample(i));
      byId("coloring-examples").appendChild(btn);
    });
    COLORING_ALGOS.forEach((_, i) => {
      const btn = document.createElement("button");
      btn.type = "button";
      btn.textContent = COLORING_ALGOS[i].title;
      btn.addEventListener("click", () => showColoringExample(i));
      byId("coloring-algos").appendChild(btn);
    });
    setColoringExample(0);
  }

  if (typeof WasmGraph !== "function" || typeof WasmGraph.prototype.bfs !== "function") {
    byId("out-tree").textContent = needRebuild;
  } else {
    const TREE_EXAMPLES = [
      { title: "4-node", n: 4, edges: [0, 1, 1, 0, 2, 1, 1, 3, 1], source: 0 },
      { title: "Path 5", n: 5, edges: [0, 1, 1, 1, 2, 1, 2, 3, 1, 3, 4, 1], source: 0 },
      { title: "Star 5", n: 5, edges: [0, 1, 1, 0, 2, 1, 0, 3, 1, 0, 4, 1], source: 0 },
      { title: "Grid 6", n: 6, edges: [0, 1, 1, 0, 2, 1, 1, 3, 1, 2, 3, 1, 2, 4, 1, 3, 5, 1, 4, 5, 1], source: 0 },
    ];
    const TREE_ALGOS = [
      { id: "bfs", title: "BFS" },
      { id: "dfsPreorder", title: "DFS preorder" },
      { id: "dfsPostorder", title: "DFS postorder" },
    ];
    let treeExampleIndex = 0;
    let treeAlgoIndex = 0;
    function showTreeExample(algoIndex) {
      treeAlgoIndex = algoIndex ?? treeAlgoIndex;
      const ex = TREE_EXAMPLES[treeExampleIndex];
      const g = buildUndirectedGraph(ex.n, ex.edges);
      const algoId = TREE_ALGOS[treeAlgoIndex].id;
      let order = [];
      let depth = [];
      let text = "Graph: " + ex.n + " nodes, source " + ex.source + "\n";
      if (algoId === "bfs") {
        const res = g.bfs(ex.source);
        order = res.getOrder();
        depth = res.getDepth();
        text += "BFS order: [" + order.join(", ") + "]\ndepth: [" + depth.map((d) => d === 4294967295 ? "∞" : d).join(", ") + "]";
      } else if (algoId === "dfsPreorder") {
        order = g.dfsPreorder(ex.source);
        text += "DFS preorder: [" + order.join(", ") + "]";
      } else {
        order = g.dfsPostorder(ex.source);
        text += "DFS postorder: [" + order.join(", ") + "]";
      }
      drawGraphTreeOnCanvas(
        byId("canvas-tree").getContext("2d"),
        byId("canvas-tree").width,
        byId("canvas-tree").height,
        ex.n,
        ex.edges,
        order,
        depth.length ? depth : null,
        ex.source
      );
      byId("out-tree").textContent = text;
      byId("tree-algos").querySelectorAll("button").forEach((b, i) => b.classList.toggle("active", i === treeAlgoIndex));
    }
    function setTreeExample(index) {
      treeExampleIndex = index;
      byId("tree-examples").querySelectorAll("button").forEach((b, i) => b.classList.toggle("active", i === index));
      showTreeExample();
    }
    TREE_EXAMPLES.forEach((_, i) => {
      const btn = document.createElement("button");
      btn.type = "button";
      btn.textContent = TREE_EXAMPLES[i].title;
      btn.addEventListener("click", () => setTreeExample(i));
      byId("tree-examples").appendChild(btn);
    });
    TREE_ALGOS.forEach((_, i) => {
      const btn = document.createElement("button");
      btn.type = "button";
      btn.textContent = TREE_ALGOS[i].title;
      btn.addEventListener("click", () => showTreeExample(i));
      byId("tree-algos").appendChild(btn);
    });
    setTreeExample(0);
  }
} catch (e) {
  const out = byId("out-graph");
  if (out) { out.className = "error"; out.textContent = "Error: " + (e.message || String(e)); }
  showError((e.message || "").toLowerCase().includes("fetch") || (e.message || "").toLowerCase().includes("import") ? needBuild + "\n\n" : "" + (e.message || String(e)));
}
