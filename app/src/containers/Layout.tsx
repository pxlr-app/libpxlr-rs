import React, { useState, useEffect, useRef, useMemo } from 'react';
import './Layout.scss';

export interface PaneProps {
	key: string,
	top: number,
	right: number,
	bottom: number,
	left: number,
	minWidth?: number,
	minHeight?: number,
	elem: React.ReactElement,
	props?: any,
}

export interface LayoutProps {
	panes: PaneProps[],
	onChange: (panes: PaneProps[]) => void,
}

interface Internal {
	bounds: DOMRect,
	dragging: boolean,
	dragId?: number,
	dragCorner?: Corner,
	dragBounds: DOMRect,
	dragSiblings?: boolean,
	lastPointerEvent?: PointerEvent,
	trottledPointerEvent?: PointerEvent,
}

export default function(props: React.PropsWithChildren<LayoutProps>) {
	const layoutRef = useRef<HTMLDivElement>(null);
	const internal = useRef<Internal>({
		bounds: new DOMRect(),
		dragging: false,
		dragBounds: new DOMRect(),
	});
	const layout = useMemo<Layout>(
		() => new Layout(props.panes),
		[props.panes]
	);

	console.log(props.panes, layout);


	const [_, render] = useState({});

	useEffect(() => {
		internal.current.bounds = layoutRef.current?.getBoundingClientRect()!;

		function onResize(_: UIEvent) {
			internal.current!.bounds = layoutRef.current?.getBoundingClientRect()!;
		}

		function onLeave(e: PointerEvent) {
			if (internal.current?.dragging) {
				internal.current = {
					...internal.current!,
					dragging: false,
					dragId: undefined,
					dragCorner: undefined,
				};

				const panes = props.panes.reduce((panes, pane) => {
					const layoutPane = layout.panes.find(p => p.id === pane.key);
					// Remove missing pane or collapsed pane
					if (layoutPane && layoutPane.width > 0 && layoutPane.height > 0) {
						pane.top = layoutPane.top;
						pane.right = layoutPane.right;
						pane.bottom = layoutPane.bottom;
						pane.left = layoutPane.left;
						panes.push(pane);
					}
					return panes;
				}, [] as PaneProps[]);


				if (props.onChange) {
					props.onChange(panes);
				} else {
					render({});
				}
			}
		}

		function onMove(e: PointerEvent) {
			if (internal.current?.dragging === true) {
				const { bounds, dragBounds, dragId, dragCorner, dragSiblings, lastPointerEvent, trottledPointerEvent } = internal.current;
				if (e.clientX === lastPointerEvent?.clientX && e.clientY === lastPointerEvent?.clientY) {
					return;
				}
				internal.current.lastPointerEvent = e;
				internal.current.trottledPointerEvent = trottledPointerEvent ?? e;

				if (dragCorner) {
					const { trottledPointerEvent } = internal.current;
					const deltaX = e.clientX - trottledPointerEvent.clientX;
					const deltaY = e.clientY - trottledPointerEvent.clientY;

					if (deltaX * deltaX + deltaY * deltaY >= 20 * 20) {
						internal.current.trottledPointerEvent = e;

						const axe = Math.abs(deltaX) > Math.abs(deltaY) ? 'vertical' : 'horizontal';
						console.log('dividing', dragId, axe);
						// TODO subdivide
					}
				}

				else {
					const edge = layout.edges[dragId!];

					const { x: oX, y: oY, width, height } = bounds;
					const [x, y] = [e.clientX - oX, e.clientY - oY];
					const [pX, pY] = [x / width * 100, y / height * 100];
					const [cX, cY] = [
						Math.max(Math.min(pX, dragBounds.right), dragBounds.left),
						Math.max(Math.min(pY, dragBounds.bottom), dragBounds.top),
					];

					const C = edge.axe === 'horizontal' ? cY : cX;
					
					edge.p = C;
					edge.updateDOM();

					if (dragSiblings) {
						for (const sibling of edge.siblings) {
							sibling.p = C;
							sibling.updateDOM();
						}
					}
				}
			}
		}

		window.addEventListener('resize', onResize);
		document.addEventListener('pointerup', onLeave);
		document.addEventListener('pointermove', onMove);
		return () => {
			window.removeEventListener('resize', onResize);
			document.removeEventListener('pointerup', onLeave);
			document.removeEventListener('pointermove', onMove);
		}
	}, [props.panes, props.onChange]);

	const onEdgeDown = (id: number) => (e: React.PointerEvent) => {
		e.preventDefault();
		e.stopPropagation();

		const edge = layout.edges[id];
		let minX: number = 0;
		let maxX: number = 100;
		let minY: number = 0;
		let maxY: number = 100;
		const breakable = edge.axe === 'horizontal'
			? Math.abs(edge.left.left - edge.right.left) < 0.1 && Math.abs(edge.left.right - edge.right.right) < 0.1
			: Math.abs(edge.left.top - edge.right.top) < 0.1 && Math.abs(edge.left.bottom - edge.right.bottom) < 0.1;

		for (const sibiling of [edge].concat(edge.siblings)) {
			minX = Math.max(minX, sibiling.left.left);
			maxX = Math.min(maxX, sibiling.right.right);
			minY = Math.max(minY, sibiling.left.top);
			maxY = Math.min(maxY, sibiling.right.bottom);
		}

		internal.current = {
			...internal.current!,
			dragging: true,
			dragId: id,
			dragBounds: new DOMRect(minX, minY, maxX - minX, maxY - minY),
			dragSiblings: !(breakable && e.ctrlKey)
		};
	};

	const onDividerDown = (id:number, corner: Corner) => (e: React.PointerEvent) => {
		e.preventDefault();
		e.stopPropagation();

		const pane = layout.panes[id];

		internal.current = {
			...internal.current!,
			dragging: true,
			dragId: id,
			dragCorner: corner,
			dragBounds: new DOMRect(pane.left, pane.top, pane.right - pane.left, pane.bottom - pane.top),
		};
	};

	const edges = layout.edges.map((edge, id) => {
		let styles: React.CSSProperties;
		if (edge.axe === 'horizontal') {
			const min = Math.max(edge.left.left, edge.right.left);
			const max = Math.min(edge.left.right, edge.right.right);
			styles = {
				top: edge.p + '%',
				left: min + '%',
				width: (max - min) + '%',
				height: 'var(--edge-size)'
			};
		} else {
			const min = Math.max(edge.left.top, edge.right.top);
			const max = Math.min(edge.left.bottom, edge.right.bottom);
			styles = {
				top: min + '%',
				left: edge.p + '%',
				width: 'var(--edge-size)',
				height: (max - min) + '%'
			};
		}

		return <div
			ref={edge.ref}
			key={`edge-${id}`}
			className={`layout-handle-edge layout-handle-edge--${edge.axe}`}
			style={styles}
			onPointerDown={onEdgeDown(id)}
		/>;
	});

	const dividers = layout.panes.map((pane, id) => <div
		key={`divider-${id}`}
		className="layout-handle"
		style={{
			top: `${pane.top}%`,
			right: `${100 - pane.right}%`,
			bottom: `${100 - pane.bottom}%`,
			left: `${pane.left}%`,
			borderWidth: pane.links.map((link, dir) => link ? (dir === 1 || dir === 2 ? `var(--border-size)` : 0) : `var(--border-size)`).join(' ')
		}}
	>
		<div key="top-left" className={`layout-handle-divider layout-handle-divider--top-left`} onPointerDown={onDividerDown(id, 'top-left')} />
		<div key="top-right" className={`layout-handle-divider layout-handle-divider--top-right`} onPointerDown={onDividerDown(id, 'top-right')} />
		<div key="bottom-left" className={`layout-handle-divider layout-handle-divider--bottom-left`} onPointerDown={onDividerDown(id, 'bottom-left')} />
		<div key="bottom-right" className={`layout-handle-divider layout-handle-divider--bottom-right`} onPointerDown={onDividerDown(id, 'bottom-right')} />
	</div>);

	const panes = layout.panes.map((pane, id) => <div
		ref={pane.ref}
		key={`pane-${id}`}
		className="layout-view-container-view"
		style={{
			top: `${pane.top}%`,
			right: `${100 - pane.right}%`,
			bottom: `${100 - pane.bottom}%`,
			left: `${pane.left}%`,
			borderWidth: pane.links.map((link, dir) => link ? (dir === 1 || dir === 2 ? `var(--border-size)` : 0) : `var(--border-size)`).join(' ')
		}}
	>
		{pane.id}
	</div>);

	return (<div className="layout" ref={layoutRef}>
		<div className="layout-edge-container">
			{edges}
		</div>
		<div className="layout-divider-container">
			{dividers}
		</div>
		<div className="layout-view-container">
			{panes}
		</div>
	</div>);
}

const { abs, min, max } = Math;

class Layout {

	public edges: Edge[];
	public panes: Pane[];

	constructor(panesProps: PaneProps[]) {
		// Find neighbors
		const neighbors: [number[], number[], number[], number[]][] = [];
		for (const pane of panesProps) {
			const neighbor: [number[], number[], number[], number[]] = [[], [], [], []];
			const pLeft = min(pane.left, pane.right);
			const pRight = max(pane.left, pane.right);
			const pTop = min(pane.top, pane.bottom);
			const pBottom = max(pane.top, pane.bottom);
			for (let i = 0, l = panesProps.length; i < l; ++i) {
				const other = panesProps[i];
				const oLeft = min(other.left, other.right);
				const oRight = max(other.left, other.right);
				const oTop = min(other.top, other.bottom);
				const oBottom = max(other.top, other.bottom);

				// TOP
				if (abs(pane.top - other.bottom) < 0.1 && segmentIntersect(pLeft, pRight, oLeft, oRight)) {
					neighbor[0].push(i);
				}
				// RIGHT
				else if (abs(pane.right - other.left) < 0.1 && segmentIntersect(pTop, pBottom, oTop, oBottom)) {
					neighbor[1].push(i);
				}
				// BOTTOM
				else if (abs(pane.bottom - other.top) < 0.1 && segmentIntersect(pLeft, pRight, oLeft, oRight)) {
					neighbor[2].push(i);
				}
				// LEFT
				else if (abs(pane.left - other.right) < 0.1 && segmentIntersect(pTop, pBottom, oTop, oBottom)) {
					neighbor[3].push(i);
				}
			}
			neighbors.push(neighbor);
		}

		// Create edges
		const edges = neighbors.reduce((edges, neighbors, a) => {
			// RIGHT, BOTTOM only
			for (let dir = 1; dir < 3; ++dir) {
				for (const b of neighbors[dir]) {
					const axe = dir % 2 ? 'vertical' : 'horizontal';
					const pos = dir % 2 ? panesProps[a].right : panesProps[a].bottom;
					const key = `${Math.min(a, b)}-${Math.max(a, b)}`;
					if (!edges.has(key)) {
						edges.set(key, new Edge(axe, pos, null!, null!, []));
					}
				}
			}
			return edges;
		}, new Map<string, Edge>());

		for (const [_, edge] of edges) {
			for (const [_, other] of edges) {
				if (edge !== other && edge.axe === other.axe && Math.abs(edge.p - other.p) < 0.1) {
					edge.siblings.push(other);
				}
			}
		}

		// Create panes
		const panes = neighbors.reduce((panes, neighbors, a) => {
			const pane = new Pane(panesProps[a].key, [[], [], [], []]);
			pane.links = neighbors.map((neighbors, dir) => {
				return neighbors.reduce((links, b) => {
					const key = `${Math.min(a, b)}-${Math.max(a, b)}`;
					if (edges.has(key)) {
						const edge = edges.get(key)!;
						edge[dir === 1 || dir === 2 ? 'left' : 'right'] = pane;
						links.push(edge);
					}
					return links;
				}, [] as Edge[]);
			}) as Links;
			panes.push(pane);
			return panes;
		}, [] as Pane[]);

		this.edges = Array.from(edges.values());
		this.panes = panes;
	}

	public dispose() {
		for (const edge of this.edges) {
			edge.dispose();
		}
		for (const pane of this.panes) {
			pane.dispose();
		}
		this.edges = undefined!;
		this.panes = undefined!;
	}
}

type Links = [Edge[], Edge[], Edge[], Edge[]];
type Corner = 'top-left' | 'top-right' | 'bottom-left' | 'bottom-right';

class Edge {
	public ref: React.RefObject<HTMLDivElement>;

	constructor(
		public axe: 'horizontal' | 'vertical',
		public p: number,
		public left: Pane,
		public right: Pane,
		public siblings: Edge[],
	) {
		this.ref = React.createRef();
	}

	public dispose() {
		this.ref = undefined!;
		this.left = undefined!;
		this.right = undefined!;
		this.siblings = undefined!;
	}

	public updateDOM() {
		if (this.ref.current) {
			if (this.axe === 'horizontal') {
				this.ref.current.style.top = this.p + '%';
			} else {
				this.ref.current.style.left = this.p + '%';
			}
		}
		this.left.updateDOM();
		this.right.updateDOM();
		for (const sibling of this.siblings) {
			sibling.left.updateDOM();
			sibling.right.updateDOM();
		}
	}
}

class Pane {
	public ref: React.RefObject<HTMLDivElement>;

	constructor(
		public id: string,
		public links: Links
	) {
		this.ref = React.createRef();
	}

	public dispose() {
		this.ref = undefined!;
		this.links = undefined!;
	}

	public updateDOM() {
		if (this.ref.current) {
			this.ref.current.style.top = `${this.top}%`;
			this.ref.current.style.right = `${100 - this.right}%`;
			this.ref.current.style.bottom = `${100 - this.bottom}%`;
			this.ref.current.style.left = `${this.left}%`;
		}
	}

	get top(): number {
		return this.links[0].length ? this.links[0][0].p : 0;
	}

	get right(): number {
		return this.links[1].length ? this.links[1][0].p : 100;
	}

	get bottom(): number {
		return this.links[2].length ? this.links[2][0].p : 100;
	}

	get left(): number {
		return this.links[3].length ? this.links[3][0].p : 0;
	}

	get width(): number {
		return this.right - this.left;
	}

	get height(): number {
		return this.bottom - this.top;
	}
}

function segmentIntersect(x1: number, x2: number, y1: number, y2: number): boolean {
	return x2 > y1 && y2 > x1;
}