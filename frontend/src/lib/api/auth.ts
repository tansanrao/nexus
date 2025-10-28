import { getJson, postJson } from "./http"
import type {
  LoginRequest,
  LoginResponse,
  LogoutRequest,
  RefreshResponse,
  SessionResponse,
  SigningKeyMetadata,
} from "./types"

export async function login(body: LoginRequest): Promise<LoginResponse> {
  return postJson<LoginResponse>("auth/login", body)
}

export async function refreshSession(): Promise<RefreshResponse> {
  return postJson<RefreshResponse>("auth/refresh")
}

export async function logout(body?: LogoutRequest): Promise<void> {
  const payload: LogoutRequest = body ?? { all_devices: false }
  await postJson("auth/logout", payload)
}

export async function getSession(): Promise<SessionResponse> {
  return postJson<SessionResponse>("auth/session")
}

export async function getSigningKeys(): Promise<SigningKeyMetadata> {
  return getJson<SigningKeyMetadata>("auth/keys")
}
