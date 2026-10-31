package com.example.zapas.features.rooms.presentation.viewmodel

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import androidx.navigation.toRoute
import androidx.paging.LoadState
import androidx.paging.PagingData
import androidx.paging.cachedIn
import androidx.paging.compose.LazyPagingItems
import com.example.zapas.features.rooms.data.dto.response.MemberDto
import com.example.zapas.core.domain.model.AppError
import com.example.zapas.features.rooms.domain.model.Room
import com.example.zapas.features.rooms.domain.usecase.DeleteRoomUseCase
import com.example.zapas.features.rooms.domain.usecase.EditRoomProfileUseCase
import com.example.zapas.features.rooms.domain.usecase.GetInviteTokenUseCase
import com.example.zapas.features.rooms.domain.usecase.GetRoomProfileUseCase
import com.example.zapas.features.rooms.domain.usecase.UpdateRoomTimingsUseCase
import com.example.zapas.core.navigation.RoomProfile
import com.example.zapas.core.utils.PagingSourceList
import com.example.zapas.features.rooms.domain.model.Member
import com.example.zapas.features.rooms.domain.usecase.GetMembersPagedUseCase
import com.example.zapas.features.rooms.domain.usecase.GetMembersUseCase
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import javax.inject.Inject

data class RoomProfileState(
    val room: Room? = null,
    val members: List<MemberDto> = emptyList(),
    val isLoading: Boolean = false,
    val error: String? = null,

    // Dialogs & Actions state
    val showEditDialog: Boolean = false,
    val showTimingsDialog: Boolean = false,
    val showInviteDialog: Boolean = false,
    val showDeleteDialog: Boolean = false,

    val isActionLoading: Boolean = false,
    val inviteToken: String? = null,
    val actionError: String? = null,
    val actionSuccessMessage: String? = null,
    val isDeleted: Boolean = false
)

@HiltViewModel
class RoomProfileViewModel @Inject constructor(
    private val savedStateHandle: SavedStateHandle,
    private val getRoomProfileUseCase: GetRoomProfileUseCase,
    private val editRoomProfileUseCase: EditRoomProfileUseCase,
    private val updateRoomTimingsUseCase: UpdateRoomTimingsUseCase,
    private val getInviteTokenUseCase: GetInviteTokenUseCase,
    getMembersPagedUseCase: GetMembersPagedUseCase,
    private val deleteRoomUseCase: DeleteRoomUseCase
) : ViewModel() {
    val members: Flow<PagingData<Member>> =
        getMembersPagedUseCase(GetMembersPagedUseCase.Params(getRoomId())).cachedIn(viewModelScope)

    private val _state = MutableStateFlow(RoomProfileState())
    val state = _state.asStateFlow()

    init {
        load()
    }

    fun getRoomId(): String = savedStateHandle.toRoute<RoomProfile>().roomId

    fun load() {
        val roomId = getRoomId()
        viewModelScope.launch {
            _state.update { it.copy(isLoading = true, error = null) }
            getRoomProfileUseCase(roomId).fold(ifLeft = { err ->
                val message = mapAppError(err)
                _state.update { it.copy(isLoading = false, error = message) }
            }, ifRight = { room ->
                _state.update { it.copy(isLoading = false, room = room) }
            })
        }
    }

    fun setShowEditDialog(show: Boolean) {
        _state.update { it.copy(showEditDialog = show, actionError = null) }
    }

    fun setShowTimingsDialog(show: Boolean) {
        _state.update { it.copy(showTimingsDialog = show, actionError = null) }
    }

    fun setShowInviteDialog(show: Boolean) {
        _state.update { it.copy(showInviteDialog = show, actionError = null) }
    }

    fun setShowDeleteDialog(show: Boolean) {
        _state.update { it.copy(showDeleteDialog = show, actionError = null) }
    }

    fun clearActionMessages() {
        _state.update { it.copy(actionError = null, actionSuccessMessage = null) }
    }

    fun editProfile(displayName: String, bio: String) {
        val roomId = getRoomId()
        viewModelScope.launch {
            _state.update { it.copy(isActionLoading = true, actionError = null) }
            editRoomProfileUseCase(
                EditRoomProfileUseCase.Params(
                    roomId = roomId,
                    displayName = displayName.ifBlank { null },
                    bio = bio.ifBlank { null })
            ).fold(ifLeft = { err ->
                _state.update {
                    it.copy(
                        isActionLoading = false, actionError = mapAppError(err)
                    )
                }
            }, ifRight = { updatedRoom ->
                _state.update {
                    it.copy(
                        isActionLoading = false,
                        room = updatedRoom,
                        showEditDialog = false,
                        actionSuccessMessage = "Профиль комнаты обновлен"
                    )
                }
            })
        }
    }

    fun updateTimings(expiredNotify: String?, reviewInterval: String?) {
        val expired = expiredNotify?.trim()?.takeIf { it.isNotEmpty() }
        val review = reviewInterval?.trim()?.takeIf { it.isNotEmpty() }
        if ((expired != null && expired.toLongOrNull() == null) ||
            (review != null && review.toLongOrNull() == null)) {
            _state.update { it.copy(actionError = "Введите целое число допустимого размера") }
            return
        }
        val roomId = getRoomId()
        viewModelScope.launch {
            _state.update { it.copy(isActionLoading = true, actionError = null) }
            updateRoomTimingsUseCase(
                UpdateRoomTimingsUseCase.Params(
                    roomId = roomId,
                    expiredNotify = expired?.toLong(),
                    reviewInterval = review?.toLong())
            ).fold(ifLeft = { err ->
                _state.update {
                    it.copy(
                        isActionLoading = false, actionError = mapAppError(err)
                    )
                }
            }, ifRight = { updatedRoom ->
                _state.update {
                    it.copy(
                        isActionLoading = false,
                        room = updatedRoom,
                        showTimingsDialog = false,
                        actionSuccessMessage = "Настройки уведомлений обновлены"
                    )
                }
            })
        }
    }

    fun generateInviteToken() {
        val roomId = getRoomId()
        viewModelScope.launch {
            _state.update { it.copy(isActionLoading = true, actionError = null) }
            getInviteTokenUseCase(roomId).fold(ifLeft = { err ->
                _state.update {
                    it.copy(
                        isActionLoading = false, actionError = mapAppError(err)
                    )
                }
            }, ifRight = { token ->
                _state.update {
                    it.copy(
                        isActionLoading = false, inviteToken = token, showInviteDialog = true
                    )
                }
            })
        }
    }

    fun deleteRoom() {
        val roomId = getRoomId()
        viewModelScope.launch {
            _state.update { it.copy(isActionLoading = true, actionError = null) }
            deleteRoomUseCase(roomId).fold(ifLeft = { err ->
                _state.update {
                    it.copy(
                        isActionLoading = false, actionError = mapAppError(err)
                    )
                }
            }, ifRight = {
                _state.update {
                    it.copy(
                        isActionLoading = false, showDeleteDialog = false, isDeleted = true
                    )
                }
            })
        }
    }

    private fun mapAppError(err: AppError): String {
        return when (err) {
            is AppError.ApiError -> err.msg
            is AppError.NetworkError -> "Ошибка сети. Проверьте соединение."
            is AppError.UnknownError -> err.msg ?: "Неизвестная ошибка"
        }
    }
}
