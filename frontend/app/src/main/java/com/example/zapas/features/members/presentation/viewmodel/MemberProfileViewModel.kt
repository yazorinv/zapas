package com.example.zapas.features.members.presentation.viewmodel

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import androidx.navigation.toRoute
import com.example.zapas.core.domain.model.AppError
import com.example.zapas.core.navigation.MemberProfile
import com.example.zapas.features.members.domain.model.MemberPortion
import com.example.zapas.features.members.domain.usecase.GetDefaultPortionsUseCase
import com.example.zapas.features.members.domain.usecase.GetMemberPortionsUseCase
import com.example.zapas.features.members.domain.usecase.GetMemberUseCase
import com.example.zapas.features.profile.domain.usecase.GetProfileUseCase
import com.example.zapas.features.rooms.domain.model.Member
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import java.text.SimpleDateFormat
import java.util.Calendar
import java.util.Locale
import javax.inject.Inject

data class MemberProfileState(
    val member: Member? = null,
    val currentUserId: String? = null,
    val defaultPortions: Long? = null,
    val portions: List<MemberPortion> = emptyList(),
    val periodTitle: String = "",
    val isLoading: Boolean = true,
    val isDefaultPortionsLoading: Boolean = true,
    val isPortionsLoading: Boolean = true,
    val error: String? = null,
    val identityError: String? = null,
    val defaultPortionsError: String? = null,
    val portionsError: String? = null
) {
    val isOwnProfile: Boolean?
        get() = currentUserId?.let { userId -> member?.userId?.let { it == userId } }
}

@HiltViewModel
class MemberProfileViewModel @Inject constructor(
    savedStateHandle: SavedStateHandle,
    private val getMemberUseCase: GetMemberUseCase,
    private val getDefaultPortionsUseCase: GetDefaultPortionsUseCase,
    private val getMemberPortionsUseCase: GetMemberPortionsUseCase,
    private val getProfileUseCase: GetProfileUseCase
) : ViewModel() {
    private val memberId = savedStateHandle.toRoute<MemberProfile>().memberId
    private val period = Calendar.getInstance().apply { set(Calendar.DAY_OF_MONTH, 1) }
    private val dateFormat = SimpleDateFormat("yyyy-MM-dd", Locale.ROOT)
    private val begin = dateFormat.format(period.time)
    private val end = dateFormat.format((period.clone() as Calendar).apply {
        set(Calendar.DAY_OF_MONTH, getActualMaximum(Calendar.DAY_OF_MONTH))
    }.time)
    private val _state = MutableStateFlow(MemberProfileState(
        periodTitle = SimpleDateFormat("LLLL yyyy", Locale("ru")).format(period.time)
    ))
    val state = _state.asStateFlow()
    private var loadJob: Job? = null

    init {
        load()
    }

    fun load() {
        loadJob?.cancel()
        _state.update {
            it.copy(
                isLoading = true, isDefaultPortionsLoading = true, isPortionsLoading = true,
                error = null, identityError = null, defaultPortionsError = null, portionsError = null
            )
        }
        loadJob = viewModelScope.launch {
            launch {
                getMemberUseCase(memberId).fold(
                    { error -> _state.update { it.copy(isLoading = false, error = formatError(error)) } },
                    { member -> _state.update { it.copy(isLoading = false, member = member) } }
                )
            }
            launch {
                getProfileUseCase().fold(
                    { error -> _state.update { it.copy(identityError = formatError(error)) } },
                    { user -> _state.update { it.copy(currentUserId = user.id) } }
                )
            }
            launch {
                getDefaultPortionsUseCase(memberId).fold(
                    { error -> _state.update {
                        it.copy(isDefaultPortionsLoading = false, defaultPortionsError = formatError(error))
                    } },
                    { portions -> _state.update {
                        it.copy(isDefaultPortionsLoading = false, defaultPortions = portions)
                    } }
                )
            }
            launch {
                getMemberPortionsUseCase(GetMemberPortionsUseCase.Params(memberId, begin, end)).fold(
                    { error -> _state.update {
                        it.copy(isPortionsLoading = false, portionsError = formatError(error))
                    } },
                    { portions -> _state.update {
                        it.copy(isPortionsLoading = false, portions = portions.sortedBy { record -> record.date })
                    } }
                )
            }
        }
    }

    private fun formatError(error: AppError): String = when (error) {
        is AppError.ApiError -> if (error.httpStatus == 404) "Участник не найден или недоступен" else error.msg
        is AppError.NetworkError -> "Ошибка сети. Проверьте подключение."
        is AppError.UnknownError -> error.msg ?: "Неизвестная ошибка"
    }
}
