import { Box, Container, Stack, Typography } from "@mui/material";
import { useApi } from "./api";

export default function App() {
	const version = useApi("meta", "version");

	return (
		<Container maxWidth="sm">
			<Stack spacing={2} sx={{ py: 8 }}>
				<Typography variant="h3" component="h1">
					Tamanu deployment setup
				</Typography>
				<Typography color="text.secondary">
					Scaffolding is in place. The wizard lands in a later phase.
				</Typography>
				<Box sx={{ fontFamily: "monospace", fontSize: 14 }}>
					{version.status === "ok" && (
						<span>
							{version.data.name} v{version.data.version}
						</span>
					)}
					{version.status === "loading" && <span>connecting…</span>}
					{version.status === "error" && (
						<span>api unreachable: {version.error.message}</span>
					)}
				</Box>
			</Stack>
		</Container>
	);
}
