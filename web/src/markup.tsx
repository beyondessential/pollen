import type { ReactNode } from "react";

// Inline markup for ruleset prose: Markdown links (which open in a new tab),
// plus **bold**, *italic*, and `code`. The ruleset is trusted, content-addressed
// authoring input, but we still render to React nodes — never raw HTML — and
// only accept http(s) link targets, so authored text can't inject markup or a
// javascript: URL.
const TOKEN = /\[([^\]]+)\]\((https?:\/\/[^\s)]+)\)|\*\*([^*]+)\*\*|`([^`]+)`|\*([^*]+)\*/g;

export function Markup({ text }: { text: string }) {
	return <>{render(text)}</>;
}

function render(text: string): ReactNode[] {
	const out: ReactNode[] = [];
	let last = 0;
	let key = 0;
	let m: RegExpExecArray | null;
	TOKEN.lastIndex = 0;
	// biome-ignore lint/suspicious/noAssignInExpressions: standard regex-exec loop
	while ((m = TOKEN.exec(text)) !== null) {
		if (m.index > last) out.push(text.slice(last, m.index));
		const [whole, linkText, href, bold, code, italic] = m;
		if (href) {
			out.push(
				<a key={key++} href={href} target="_blank" rel="noopener noreferrer">
					{linkText}
				</a>,
			);
		} else if (bold) {
			out.push(<strong key={key++}>{bold}</strong>);
		} else if (code) {
			out.push(<code key={key++}>{code}</code>);
		} else if (italic) {
			out.push(<em key={key++}>{italic}</em>);
		}
		last = m.index + whole.length;
	}
	if (last < text.length) out.push(text.slice(last));
	return out;
}
