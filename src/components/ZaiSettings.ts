import { invoke } from "@tauri-apps/api/core";

let modalElement: HTMLElement | null = null;

// Function to refresh Z.ai UI - will be attached by main.ts
let refreshZaiUI: (() => Promise<void>) | null = null;

export function setRefreshZaiUI(fn: () => Promise<void>): void {
	refreshZaiUI = fn;
}

export function openZaiModal(isConnected: boolean): void {
	openModal(isConnected);
}

export async function checkZaiApiKey(): Promise<boolean> {
	return await invoke<boolean>("check_zai_api_key");
}

export function createZaiConnectionBadge(isConnected: boolean): HTMLElement {
	const container = document.createElement("button");
	container.className = isConnected ? "zai-header-badge zai-header-badge-connected" : "zai-header-badge zai-header-badge-disconnected";

	const icon = document.createElement("span");
	icon.className = "zai-header-badge-icon";

	if (isConnected) {
		icon.innerHTML = `<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"></polyline></svg>`;
	} else {
		icon.innerHTML = `<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="12" y1="5" x2="12" y2="19"></line><line x1="5" y1="12" x2="19" y2="12"></line></svg>`;
	}

	const label = document.createElement("span");
	label.className = "zai-header-badge-label";
	label.textContent = isConnected ? "Connected" : "Connect";

	container.appendChild(icon);
	container.appendChild(label);

	container.addEventListener("click", () => openModal(isConnected));

	return container;
}

export async function createZaiSettings(): Promise<HTMLElement> {
	const settings = document.createElement("div");
	settings.className = "zai-settings";

	const hasApiKey = await invoke<boolean>("check_zai_api_key");

	if (!hasApiKey) {
		settings.style.display = "none";
	}

	return settings;
}

function openModal(isConnected: boolean): void {
	if (modalElement) return;

	const backdrop = document.createElement("div");
	backdrop.className = "modal-backdrop";

	modalElement = document.createElement("div");
	modalElement.className = "zai-modal";

	const header = document.createElement("div");
	header.className = "zai-modal-header";
	header.innerHTML = `
		<h2 class="zai-modal-title">${isConnected ? "Manage z.ai API Key" : "Connect z.ai Coding Plan"}</h2>
		<button class="zai-modal-close" aria-label="Close modal">
			<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18"></line><line x1="6" y1="6" x2="18" y2="18"></line></svg>
		</button>
	`;

	const content = document.createElement("div");
	content.className = "zai-modal-content";

	if (isConnected) {
		content.appendChild(createConnectedState());
	} else {
		content.appendChild(createInputState());
	}

	modalElement.appendChild(header);
	modalElement.appendChild(content);

	backdrop.appendChild(modalElement);
	document.body.appendChild(backdrop);

	backdrop.addEventListener("click", (e) => {
		if (e.target === backdrop) closeModal();
	});

	header.querySelector(".zai-modal-close")?.addEventListener("click", (e) => {
		e.stopPropagation();
		closeModal();
	});
}

function closeModal(): void {
	if (!modalElement) return;

	const backdrop = modalElement.parentElement;
	if (backdrop) {
		// Prevent any pointer events during close animation
		backdrop.style.pointerEvents = "none";
		backdrop.style.animation = "fade-out 0.15s ease-out forwards";

		requestAnimationFrame(() => {
			setTimeout(() => {
				backdrop.remove();
				modalElement = null;
			}, 150);
		});
	}
}

function createConnectedState(): HTMLElement {
	const container = document.createElement("div");
	container.className = "zai-modal-connected";

	const info = document.createElement("div");
	info.className = "zai-modal-info";
	info.innerHTML = `
		<div class="zai-modal-info-text">
			<div class="zai-modal-info-title">API Key Configured</div>
			<div class="zai-modal-info-desc">Your Z.ai API key is set up and ready to use.</div>
		</div>
	`;

	const actions = document.createElement("div");
	actions.className = "zai-modal-actions";

	const cancelButton = document.createElement("button");
	cancelButton.className = "btn btn-ghost";
	cancelButton.textContent = "Cancel";
	cancelButton.addEventListener("click", closeModal);

	const updateButton = document.createElement("button");
	updateButton.className = "btn btn-primary";
	updateButton.textContent = "Update";
	updateButton.addEventListener("click", () => {
		const content = modalElement?.querySelector(".zai-modal-content");
		if (content) {
			content.innerHTML = "";
			content.appendChild(createInputState());
		}
	});

	const deleteButton = document.createElement("button");
	deleteButton.className = "btn btn-destructive";
	deleteButton.textContent = "Remove";
	deleteButton.addEventListener("click", async () => {
		try {
			await invoke("delete_zai_api_key");
			closeModal();
			if (refreshZaiUI) {
				await refreshZaiUI();
			} else {
				// Fallback to reload if refresh function not available
				window.location.reload();
			}
		} catch (error) {
			console.error("Failed to delete API key:", error);
		}
	});

	actions.appendChild(updateButton);
	actions.appendChild(deleteButton);
	actions.appendChild(cancelButton);

	container.appendChild(info);
	container.appendChild(actions);

	return container;
}

function createInputState(): HTMLElement {
	const container = document.createElement("div");
	container.className = "zai-modal-input";

	const info = document.createElement("div");
	info.className = "zai-modal-info";
	info.innerHTML = `
		<div class="zai-modal-info-text">
			<div class="zai-modal-info-title">Enter Your API Key</div>
			<div class="zai-modal-info-desc">Get your API key from <a href="https://z.ai/manage-apikey/apikey-list" target="_blank" class="zai-link">z.ai/manage-apikey</a></div>
		</div>
	`;

	const inputContainer = document.createElement("div");
	inputContainer.className = "zai-modal-input-container";

	const input = document.createElement("input");
	input.type = "password";
	input.placeholder = "Enter your API key";
	input.className = "zai-modal-input-field";
	input.autofocus = true;

	const errorElement = document.createElement("div");
	errorElement.className = "zai-modal-error";
	errorElement.style.display = "none";

	const actions = document.createElement("div");
	actions.className = "zai-modal-actions";

	const saveButton = document.createElement("button");
	saveButton.className = "btn btn-primary";
	saveButton.textContent = "Save";

	const cancelButton = document.createElement("button");
	cancelButton.className = "btn btn-ghost";
	cancelButton.textContent = "Cancel";
	cancelButton.addEventListener("click", closeModal);

	const setValidationState = (validating: boolean, error?: string) => {
		saveButton.disabled = validating;
		saveButton.textContent = validating ? "Validating..." : "Save";
		input.disabled = validating;
		cancelButton.disabled = validating;

		if (error) {
			errorElement.textContent = error;
			errorElement.style.display = "block";
		} else {
			errorElement.style.display = "none";
		}
	};

	saveButton.addEventListener("click", async () => {
		const apiKey = input.value.trim();
		if (!apiKey) {
			setValidationState(false, "Please enter an API key");
			return;
		}

		setValidationState(true);

		try {
			await invoke("validate_zai_api_key", { apiKey });
			await invoke("save_zai_api_key", { apiKey });
			closeModal();
			if (refreshZaiUI) {
				await refreshZaiUI();
			} else {
				window.location.reload();
			}
		} catch (error) {
			setValidationState(false, String(error));
		}
	});

	input.addEventListener("keydown", (e) => {
		if (e.key === "Enter" && !saveButton.disabled) {
			saveButton.click();
		}
	});

	input.addEventListener("input", () => {
		if (errorElement.style.display !== "none") {
			setValidationState(false);
		}
	});

	inputContainer.appendChild(input);
	inputContainer.appendChild(errorElement);
	actions.appendChild(saveButton);
	actions.appendChild(cancelButton);

	container.appendChild(info);
	container.appendChild(inputContainer);
	container.appendChild(actions);

	return container;
}

