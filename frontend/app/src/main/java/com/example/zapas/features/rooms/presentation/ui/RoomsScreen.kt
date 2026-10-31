package com.example.zapas.features.rooms.presentation.ui

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Person
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.FloatingActionButton
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.paging.compose.collectAsLazyPagingItems
import com.example.zapas.core.utils.DefaultPagingSourceList
import com.example.zapas.core.utils.PagingSourceList
import com.example.zapas.features.rooms.domain.model.Room
import com.example.zapas.features.rooms.presentation.components.JoinRoomDialog
import com.example.zapas.features.rooms.presentation.components.RoomItem
import com.example.zapas.features.rooms.presentation.components.RoomsPagingSourceListMock
import com.example.zapas.features.rooms.presentation.viewmodel.RoomsUiState
import com.example.zapas.features.rooms.presentation.viewmodel.RoomsViewModel

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun RoomsScreen(
    onNavigateToProfile: () -> Unit,
    onNavigateToCreateRoom: () -> Unit,
    onNavigateToRoom: (String) -> Unit = {},
    viewModel: RoomsViewModel = hiltViewModel()
) {
    val pagingItems = viewModel.rooms.collectAsLazyPagingItems()
    val uiState by viewModel.uiState.collectAsState()

    val rooms = remember(pagingItems) { DefaultPagingSourceList(pagingItems) }

    RoomScreenContent(
        items = rooms,
        uiState = uiState,
        onNavigateToProfile = onNavigateToProfile,
        onNavigateToCreateRoom = onNavigateToCreateRoom,
        onNavigateToRoom = onNavigateToRoom,
        onShowDialog = { viewModel.setShowJoinDialog(true) },
        onDismissDialog = { viewModel.setShowJoinDialog(false) },
        onSetInviteToken = viewModel::setInviteTokenInput,
        onJoinRoom = { token -> viewModel.joinRoom(token, onNavigateToRoom) }
    )
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun RoomScreenContent(
    items: PagingSourceList<Room>,
    uiState: RoomsUiState,
    onNavigateToProfile: () -> Unit,
    onNavigateToCreateRoom: () -> Unit,
    onNavigateToRoom: (String) -> Unit = {},
    onShowDialog: () -> Unit = {},
    onDismissDialog: () -> Unit = {},
    onSetInviteToken: (String) -> Unit = {},
    onJoinRoom: (String) -> Unit = {}
) {
    Scaffold(topBar = {
        TopAppBar(title = { Text("Мои комнаты") }, actions = {
            IconButton(onClick = onNavigateToProfile) {
                Icon(Icons.Default.Person, contentDescription = "Профиль")
            }
        })
    }, floatingActionButton = {
        FloatingActionButton(onClick = onShowDialog) {
            Icon(Icons.Default.Add, "Присоединиться к комнате")
        }
    }) { paddingValues ->
        if (uiState.showJoinDialog) {
            JoinRoomDialog(
                uiState = uiState,
                onDismiss = onDismissDialog,
                onSetInviteToken = onSetInviteToken,
                onJoinRoom = onJoinRoom,
                onCreateRoomClick = {
                    onDismissDialog()
                    onNavigateToCreateRoom()
                }
            )
        }

        if (items.itemCount == 0 && !items.isRefreshing) {
            Box(
                modifier = Modifier
                    .fillMaxSize()
                    .padding(paddingValues),
                contentAlignment = Alignment.Center
            ) {
                Text(
                    text = "У вас пока нет комнат",
                    style = MaterialTheme.typography.bodyLarge,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
            }
        } else {
            LazyColumn(
                modifier = Modifier
                    .fillMaxSize()
                    .padding(paddingValues)
            ) {
                items(
                    count = items.itemCount
                ) { index ->

                    val room = items[index]

                    if (room != null) {
                        RoomItem(
                            room = room,
                            onClick = { onNavigateToRoom(room.id) }
                        )
                        HorizontalDivider(
                            modifier = Modifier.padding(horizontal = 16.dp),
                            thickness = 0.5.dp,
                            color = MaterialTheme.colorScheme.outlineVariant
                        )
                    }
                }

                if (items.isLoadingMore) {
                    item {
                        Box(
                            modifier = Modifier
                                .fillMaxWidth()
                                .padding(16.dp),
                            contentAlignment = Alignment.Center
                        ) {
                            CircularProgressIndicator()
                        }
                    }
                }
            }
        }
    }
}

@Preview
@Composable
fun RoomScreenPreview() {
    RoomScreenContent(
        items = RoomsPagingSourceListMock(
        (1..100).map {
            Room(
                bio = "bio",
                createdAt = "2006",
                displayName = "Room${it}",
                id = it.toString(),
                expiredNotify = 86400L,
                reviewInterval = 2592000L
            )
        }),
        uiState = RoomsUiState(),
        onNavigateToProfile = {},
        onNavigateToCreateRoom = {}
    )
}
