package com.example.zapas.core.navigation

import androidx.lifecycle.ViewModel
import com.example.zapas.core.local.TokenManager
import dagger.hilt.android.lifecycle.HiltViewModel
import javax.inject.Inject

@HiltViewModel
class SessionViewModel @Inject constructor(private val tokenManager: TokenManager) : ViewModel() {
    val sessionExpired = tokenManager.sessionExpired
    fun acknowledgeSessionExpiry() = tokenManager.acknowledgeSessionExpiry()
}
