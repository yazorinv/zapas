package com.example.zapas.core.network

import arrow.core.Either
import com.example.zapas.core.domain.model.AppError
import com.example.zapas.core.local.TokenManager
import com.example.zapas.features.auth.domain.repository.AuthRepository
import dagger.Lazy
import kotlinx.coroutines.runBlocking
import okhttp3.Authenticator
import okhttp3.Request
import okhttp3.Response
import okhttp3.Route
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class TokenAuthenticator @Inject constructor(
    private val tokenManager: TokenManager, private val authRepository: Lazy<AuthRepository>
) : Authenticator {

    @Synchronized
    override fun authenticate(route: Route?, response: Response): Request? {
        val tokenInRequest = response.request.header("Authorization")
        val current = runBlocking { tokenManager.getBasicToken()?.value } ?: return null
        if (tokenInRequest != "Bearer $current") {
            return response.request.newBuilder().header("Authorization", "Bearer $current").build()
        }

        if (response.priorResponse?.code == 401) {
            runBlocking { tokenManager.clearTokens(sessionExpired = true) }
            return null
        }

        val refreshed = runBlocking { authRepository.get().prolong() }
        if (refreshed is Either.Left) {
            if ((refreshed.value as? AppError.ApiError)?.httpStatus == 401)
                runBlocking { tokenManager.clearTokens(sessionExpired = true) }
            return null
        }

        val newToken = runBlocking { tokenManager.getBasicToken()?.value } ?: run {
            runBlocking { tokenManager.clearTokens(sessionExpired = true) }
            return null
        }
        return response.request.newBuilder().header("Authorization", "Bearer $newToken").build()
    }
}