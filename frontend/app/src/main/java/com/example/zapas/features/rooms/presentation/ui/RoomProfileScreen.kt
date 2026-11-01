package com.example.zapas.features.rooms.presentation.ui

import android.content.res.Configuration
import android.widget.Toast
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.DateRange
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material.icons.filled.Edit
import androidx.compose.material.icons.filled.Home
import androidx.compose.material.icons.filled.Info
import androidx.compose.material.icons.filled.MoreVert
import androidx.compose.material.icons.filled.Notifications
import androidx.compose.material.icons.filled.Person
import androidx.compose.material.icons.filled.Refresh
import androidx.compose.material.icons.filled.Share
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.ListItem
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedCard
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.TopAppBar
import androidx.compose.material3.TopAppBarDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.platform.LocalClipboardManager
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.paging.compose.collectAsLazyPagingItems
import com.example.zapas.core.components.PreviewableAlertDialog
import com.example.zapas.core.utils.DefaultPagingSourceList
import com.example.zapas.core.utils.PagingSourceList
import com.example.zapas.features.rooms.data.dto.response.MemberDto
import com.example.zapas.features.rooms.domain.model.Member
import com.example.zapas.features.rooms.domain.model.Room
import com.example.zapas.features.rooms.presentation.viewmodel.RoomProfileState
import com.example.zapas.features.rooms.presentation.viewmodel.RoomProfileViewModel
import com.example.zapas.ui.theme.ZapasTheme

@Composable
fun RoomProfileScreen(
    onBack: () -> Unit,
    onNavigateToMember: (String) -> Unit,
    viewModel: RoomProfileViewModel = hiltViewModel()
) {
    val state by viewModel.state.collectAsState()
    val snackbarHostState = remember { SnackbarHostState() }

    val memberPagingItems = viewModel.members.collectAsLazyPagingItems()
    val members = remember(memberPagingItems) { DefaultPagingSourceList(memberPagingItems) }

    LaunchedEffect(state.isDeleted) {
        if (state.isDeleted) {
            onBack()
        }
    }

    LaunchedEffect(state.actionSuccessMessage, state.actionError) {
        state.actionSuccessMessage?.let { msg ->
            snackbarHostState.showSnackbar(msg)
            viewModel.clearActionMessages()
        }
        state.actionError?.let { err ->
            snackbarHostState.showSnackbar(err)
            viewModel.clearActionMessages()
        }
    }

    RoomProfileScreenContent(
        members = members,
        state = state,
        onBack = onBack,
        onRefresh = viewModel::load,
        onShowEditDialog = { viewModel.setShowEditDialog(true) },
        onShowTimingsDialog = { viewModel.setShowTimingsDialog(true) },
        onGenerateInvite = viewModel::generateInviteToken,
        onShowDeleteDialog = { viewModel.setShowDeleteDialog(true) },
        onNavigateToMember = onNavigateToMember
    )

    // Dialogs
    if (state.showEditDialog && state.room != null) {
        EditRoomProfileDialog(
            currentName = state.room?.displayName.orEmpty(),
            currentBio = state.room?.bio.orEmpty(),
            isLoading = state.isActionLoading,
            onDismiss = { viewModel.setShowEditDialog(false) },
            onConfirm = viewModel::editProfile
        )
    }

    if (state.showTimingsDialog && state.room != null) {
        UpdateTimingsDialog(
            currentExpiredNotify = state.room?.expiredNotify?.toString().orEmpty(),
            currentReviewInterval = state.room?.reviewInterval?.toString().orEmpty(),
            isLoading = state.isActionLoading,
            onDismiss = { viewModel.setShowTimingsDialog(false) },
            onConfirm = viewModel::updateTimings
        )
    }

    if (state.showInviteDialog) {
        InviteTokenDialog(
            token = state.inviteToken.orEmpty(),
            onDismiss = { viewModel.setShowInviteDialog(false) }
        )
    }

    if (state.showDeleteDialog) {
        DeleteRoomDialog(
            isLoading = state.isActionLoading,
            onDismiss = { viewModel.setShowDeleteDialog(false) },
            onConfirm = viewModel::deleteRoom
        )
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun RoomProfileScreenContent(
    members: PagingSourceList<Member>,
    state: RoomProfileState,
    onBack: () -> Unit,
    onRefresh: () -> Unit,
    onShowEditDialog: () -> Unit,
    onShowTimingsDialog: () -> Unit,
    onGenerateInvite: () -> Unit,
    onShowDeleteDialog: () -> Unit,
    onNavigateToMember: (String) -> Unit = {}
) {
    var showMenu by remember { mutableStateOf(false) }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Профиль комнаты") },
                navigationIcon = {
                    IconButton(onClick = onBack) {
                        Icon(
                            imageVector = Icons.AutoMirrored.Filled.ArrowBack,
                            contentDescription = "Назад"
                        )
                    }
                },
                actions = {
                    IconButton(onClick = onRefresh) {
                        Icon(Icons.Default.Refresh, contentDescription = "Обновить")
                    }
                    Box {
                        IconButton(onClick = { showMenu = true }) {
                            Icon(Icons.Default.MoreVert, contentDescription = "Еще")
                        }
                        DropdownMenu(
                            expanded = showMenu,
                            onDismissRequest = { showMenu = false }
                        ) {
                            DropdownMenuItem(
                                text = { Text("Редактировать") },
                                onClick = {
                                    showMenu = false
                                    onShowEditDialog()
                                },
                                leadingIcon = {
                                    Icon(
                                        Icons.Default.Edit,
                                        contentDescription = null
                                    )
                                }
                            )
                            DropdownMenuItem(
                                text = { Text("Настройки времени") },
                                onClick = {
                                    showMenu = false
                                    onShowTimingsDialog()
                                },
                                leadingIcon = {
                                    Icon(
                                        Icons.Default.DateRange,
                                        contentDescription = null
                                    )
                                }
                            )
                            DropdownMenuItem(
                                text = { Text("Код приглашения") },
                                onClick = {
                                    showMenu = false
                                    onGenerateInvite()
                                },
                                leadingIcon = { Icon(Icons.Default.Add, contentDescription = null) }
                            )
                            HorizontalDivider()
                            DropdownMenuItem(
                                text = {
                                    Text(
                                        "Удалить комнату",
                                        color = MaterialTheme.colorScheme.error
                                    )
                                },
                                onClick = {
                                    showMenu = false
                                    onShowDeleteDialog()
                                },
                                leadingIcon = {
                                    Icon(
                                        Icons.Default.Delete,
                                        contentDescription = null,
                                        tint = MaterialTheme.colorScheme.error
                                    )
                                }
                            )
                        }
                    }
                },
                colors = TopAppBarDefaults.topAppBarColors(
                    containerColor = MaterialTheme.colorScheme.surface
                )
            )
        }
    ) { paddingValues ->
        Box(
            modifier = Modifier
                .fillMaxSize()
                .padding(paddingValues)
        ) {
            when {
                state.isLoading && state.room == null -> {
                    CircularProgressIndicator(modifier = Modifier.align(Alignment.Center))
                }

                state.error != null && state.room == null -> {
                    Column(
                        modifier = Modifier
                            .align(Alignment.Center)
                            .padding(24.dp),
                        horizontalAlignment = Alignment.CenterHorizontally,
                        verticalArrangement = Arrangement.spacedBy(16.dp)
                    ) {
                        Icon(
                            imageVector = Icons.Default.Info,
                            contentDescription = null,
                            modifier = Modifier.size(48.dp),
                            tint = MaterialTheme.colorScheme.error
                        )
                        Text(
                            text = state.error,
                            style = MaterialTheme.typography.bodyLarge,
                            color = MaterialTheme.colorScheme.error,
                            textAlign = TextAlign.Center
                        )
                        Button(onClick = onRefresh) {
                            Text("Повторить")
                        }
                    }
                }

                state.room != null -> {
                    val room = state.room
                    val clipboardManager = LocalClipboardManager.current
                    val context = LocalContext.current

                    Column(
                        modifier = Modifier
                            .fillMaxSize()
                            .verticalScroll(rememberScrollState())
                            .padding(16.dp),
                        verticalArrangement = Arrangement.spacedBy(16.dp)
                    ) {
                        // Room Header Card
                        RoomHeaderCard(
                            room = room,
                            onCopyId = {
                                clipboardManager.setText(AnnotatedString(room.id))
                                Toast.makeText(context, "ID комнаты скопирован", Toast.LENGTH_SHORT)
                                    .show()
                            }
                        )

                        // Quick Actions Bar
                        QuickActionsBar(
                            onGenerateInvite = onGenerateInvite,
                            onEdit = onShowEditDialog,
                            onTimings = onShowTimingsDialog
                        )

                        // Room Bio / Description Card
                        RoomBioCard(bio = room.bio)

                        // Room Settings / Timings Card
                        RoomTimingsCard(
                            expiredNotify = room.expiredNotify,
                            reviewInterval = room.reviewInterval,
                            onEditTimings = onShowTimingsDialog
                        )

                        // Room Members Section
                        RoomMembersSection(members, onNavigateToMember)

                        Spacer(modifier = Modifier.height(8.dp))

                        // Delete Room Button
                        OutlinedButton(
                            onClick = onShowDeleteDialog,
                            modifier = Modifier.fillMaxWidth(),
                            colors = ButtonDefaults.outlinedButtonColors(
                                contentColor = MaterialTheme.colorScheme.error
                            )
                        ) {
                            Icon(Icons.Default.Delete, contentDescription = null)
                            Spacer(modifier = Modifier.width(8.dp))
                            Text("Удалить комнату")
                        }

                        Spacer(modifier = Modifier.height(16.dp))
                    }
                }
            }
        }
    }
}

@Composable
fun RoomHeaderCard(
    room: Room,
    onCopyId: () -> Unit
) {
    Card(
        modifier = Modifier.fillMaxWidth(),
        colors = CardDefaults.cardColors(
            containerColor = MaterialTheme.colorScheme.primaryContainer.copy(alpha = 0.4f)
        )
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(20.dp),
            horizontalAlignment = Alignment.CenterHorizontally
        ) {
            Box(
                modifier = Modifier
                    .size(80.dp)
                    .clip(CircleShape)
                    .background(MaterialTheme.colorScheme.primary),
                contentAlignment = Alignment.Center
            ) {
                Icon(
                    imageVector = Icons.Default.Home,
                    contentDescription = null,
                    modifier = Modifier.size(40.dp),
                    tint = MaterialTheme.colorScheme.onPrimary
                )
            }

            Spacer(modifier = Modifier.height(12.dp))

            Text(
                text = room.displayName,
                style = MaterialTheme.typography.headlineSmall,
                fontWeight = FontWeight.Bold,
                textAlign = TextAlign.Center
            )

            Spacer(modifier = Modifier.height(8.dp))

            // Room ID Pill
            Surface(
                onClick = onCopyId,
                shape = RoundedCornerShape(16.dp),
                color = MaterialTheme.colorScheme.surfaceVariant
            ) {
                Row(
                    modifier = Modifier.padding(horizontal = 12.dp, vertical = 6.dp),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(6.dp)
                ) {
                    Text(
                        text = "ID: ${room.id}",
                        style = MaterialTheme.typography.labelMedium,
                        fontFamily = FontFamily.Monospace,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis
                    )
                    Icon(
                        imageVector = Icons.Default.Share,
                        contentDescription = "Копировать ID",
                        modifier = Modifier.size(14.dp),
                        tint = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                }
            }

            if (room.createdAt.isNotBlank()) {
                Spacer(modifier = Modifier.height(8.dp))
                Row(
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(4.dp)
                ) {
                    Icon(
                        imageVector = Icons.Default.DateRange,
                        contentDescription = null,
                        modifier = Modifier.size(14.dp),
                        tint = MaterialTheme.colorScheme.outline
                    )
                    Text(
                        text = "Создана: ${room.createdAt}",
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.outline
                    )
                }
            }
        }
    }
}

@Composable
fun QuickActionsBar(
    onGenerateInvite: () -> Unit,
    onEdit: () -> Unit,
    onTimings: () -> Unit
) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.spacedBy(8.dp)
    ) {
        Button(
            onClick = onGenerateInvite,
            modifier = Modifier.weight(1f)
        ) {
            Icon(Icons.Default.Share, contentDescription = null, modifier = Modifier.size(18.dp))
            Spacer(modifier = Modifier.width(6.dp))
            Text("Инвайт", style = MaterialTheme.typography.labelLarge)
        }

        OutlinedButton(
            onClick = onEdit,
            modifier = Modifier.weight(1f)
        ) {
            Icon(Icons.Default.Edit, contentDescription = null, modifier = Modifier.size(18.dp))
            Spacer(modifier = Modifier.width(6.dp))
            Text("Инфо", style = MaterialTheme.typography.labelLarge)
        }

        OutlinedButton(
            onClick = onTimings,
            modifier = Modifier.weight(1f)
        ) {
            Icon(
                Icons.Default.DateRange,
                contentDescription = null,
                modifier = Modifier.size(18.dp)
            )
            Spacer(modifier = Modifier.width(6.dp))
            Text("Тайминги", style = MaterialTheme.typography.labelLarge)
        }
    }
}

@Composable
fun RoomBioCard(bio: String) {
    Card(
        modifier = Modifier.fillMaxWidth(),
        colors = CardDefaults.cardColors(
            containerColor = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.5f)
        )
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp)
        ) {
            Text(
                text = "Описание",
                style = MaterialTheme.typography.titleSmall,
                fontWeight = FontWeight.SemiBold,
                color = MaterialTheme.colorScheme.primary
            )
            Spacer(modifier = Modifier.height(8.dp))
            Text(
                text = if (bio.isNotBlank()) bio else "Описание комнаты не указано",
                style = MaterialTheme.typography.bodyMedium,
                color = if (bio.isNotBlank()) MaterialTheme.colorScheme.onSurface else MaterialTheme.colorScheme.outline
            )
        }
    }
}

@Composable
fun RoomTimingsCard(
    expiredNotify: Long?,
    reviewInterval: Long?,
    onEditTimings: () -> Unit
) {
    OutlinedCard(
        modifier = Modifier.fillMaxWidth()
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp)
        ) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                Text(
                    text = "Параметры уведомлений",
                    style = MaterialTheme.typography.titleSmall,
                    fontWeight = FontWeight.SemiBold,
                    color = MaterialTheme.colorScheme.primary
                )
                IconButton(onClick = onEditTimings, modifier = Modifier.size(24.dp)) {
                    Icon(
                        imageVector = Icons.Default.Edit,
                        contentDescription = "Редактировать тайминги",
                        tint = MaterialTheme.colorScheme.primary
                    )
                }
            }

            Spacer(modifier = Modifier.height(12.dp))

            Row(
                modifier = Modifier.fillMaxWidth(),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(12.dp)
            ) {
                Icon(
                    imageVector = Icons.Default.Notifications,
                    contentDescription = null,
                    tint = MaterialTheme.colorScheme.secondary
                )
                Column {
                    Text(
                        text = "Уведомление об истечении срока",
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.outline
                    )
                    Text(
                        text = expiredNotify?.let { "$it с" } ?: "Не установлено",
                        style = MaterialTheme.typography.bodyMedium,
                        fontWeight = FontWeight.Medium
                    )
                }
            }

            Spacer(modifier = Modifier.height(12.dp))

            Row(
                modifier = Modifier.fillMaxWidth(),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(12.dp)
            ) {
                Icon(
                    imageVector = Icons.Default.DateRange,
                    contentDescription = null,
                    tint = MaterialTheme.colorScheme.secondary
                )
                Column {
                    Text(
                        text = "Интервал инвентаризации / проверки",
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.outline
                    )
                    Text(
                        text = reviewInterval?.let { "$it с" } ?: "Не установлено",
                        style = MaterialTheme.typography.bodyMedium,
                        fontWeight = FontWeight.Medium
                    )
                }
            }
        }
    }
}

@Composable
fun RoomMembersSection(
    members: PagingSourceList<Member>,
    onNavigateToMember: (String) -> Unit
) {
    Column(
        modifier = Modifier.fillMaxWidth(),
        verticalArrangement = Arrangement.spacedBy(8.dp)
    ) {
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically
        ) {
            Text(
                text = "Участники (${members.itemCount})",
                style = MaterialTheme.typography.titleMedium,
                fontWeight = FontWeight.Bold
            )
            if (members.isRefreshing) {
                CircularProgressIndicator(modifier = Modifier.size(18.dp), strokeWidth = 2.dp)
            }
        }

        if (members.itemCount == 0 && !members.isRefreshing) {
            Card(
                modifier = Modifier.fillMaxWidth(),
                colors = CardDefaults.cardColors(
                    containerColor = MaterialTheme.colorScheme.surfaceContainer
                )
            ) {
                Text(
                    text = "Список участников пуст",
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.outline,
                    modifier = Modifier.padding(16.dp),
                    textAlign = TextAlign.Center
                )
            }
        } else {
            Card(
                modifier = Modifier.fillMaxWidth()
            ) {
                Column {
                    repeat(members.itemCount) { index ->
                        val member = members[index]

                        if (member != null) {
                            ListItem(
                                modifier = Modifier.clickable { onNavigateToMember(member.id) },
                                headlineContent = {
                                    Text(
                                        text = member.displayName.ifBlank { "Без имени" },
                                        fontWeight = FontWeight.Medium
                                    )
                                },
                                supportingContent = if (member.bio.isNotBlank()) {
                                    {
                                        Text(
                                            member.bio,
                                            maxLines = 1,
                                            overflow = TextOverflow.Ellipsis
                                        )
                                    }
                                } else null,
                                leadingContent = {
                                    Box(
                                        modifier = Modifier
                                            .size(40.dp)
                                            .clip(CircleShape)
                                            .background(MaterialTheme.colorScheme.secondaryContainer),
                                        contentAlignment = Alignment.Center
                                    ) {
                                        Icon(
                                            imageVector = Icons.Default.Person,
                                            contentDescription = null,
                                            tint = MaterialTheme.colorScheme.onSecondaryContainer
                                        )
                                    }
                                }
                            )
                            if (index < members.itemCount - 1) {
                                HorizontalDivider(
                                    modifier = Modifier.padding(horizontal = 16.dp),
                                    color = MaterialTheme.colorScheme.outlineVariant
                                )
                            }
                        }
                    }
                }
            }
        }
    }
}

@Composable
fun EditRoomProfileDialog(
    currentName: String,
    currentBio: String,
    isLoading: Boolean,
    onDismiss: () -> Unit,
    onConfirm: (String, String) -> Unit
) {
    var name by remember { mutableStateOf(currentName) }
    var bio by remember { mutableStateOf(currentBio) }

    PreviewableAlertDialog(
        onDismissRequest = onDismiss,
        title = { Text("Редактировать комнату") },
        text = {
            Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                OutlinedTextField(
                    value = name,
                    onValueChange = { name = it },
                    label = { Text("Название комнаты") },
                    singleLine = true,
                    enabled = !isLoading,
                    modifier = Modifier.fillMaxWidth()
                )
                OutlinedTextField(
                    value = bio,
                    onValueChange = { bio = it },
                    label = { Text("Описание") },
                    minLines = 3,
                    maxLines = 5,
                    enabled = !isLoading,
                    modifier = Modifier.fillMaxWidth()
                )
            }
        },
        confirmButton = {
            Button(
                onClick = { onConfirm(name, bio) },
                enabled = !isLoading && name.isNotBlank()
            ) {
                if (isLoading) {
                    CircularProgressIndicator(
                        modifier = Modifier.size(18.dp),
                        color = MaterialTheme.colorScheme.onPrimary,
                        strokeWidth = 2.dp
                    )
                } else {
                    Text("Сохранить")
                }
            }
        },
        dismissButton = {
            TextButton(onClick = onDismiss, enabled = !isLoading) {
                Text("Отмена")
            }
        }
    )
}

@Composable
fun UpdateTimingsDialog(
    currentExpiredNotify: String,
    currentReviewInterval: String,
    isLoading: Boolean,
    onDismiss: () -> Unit,
    onConfirm: (String, String) -> Unit
) {
    var expiredNotify by remember { mutableStateOf(currentExpiredNotify) }
    var reviewInterval by remember { mutableStateOf(currentReviewInterval) }
    val expiredValid = expiredNotify.isBlank() || expiredNotify.trim().toLongOrNull() != null
    val reviewValid = reviewInterval.isBlank() || reviewInterval.trim().toLongOrNull() != null

    PreviewableAlertDialog(
        onDismissRequest = onDismiss,
        title = { Text("Настройки времени") },
        text = {
            Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                OutlinedTextField(
                    value = expiredNotify,
                    onValueChange = { expiredNotify = it },
                    label = { Text("Уведомление об истечении, с") },
                    isError = !expiredValid,
                    supportingText = { if (!expiredValid) Text("Введите целое число допустимого размера") },
                    placeholder = { Text("Число секунд") },
                    singleLine = true,
                    enabled = !isLoading,
                    modifier = Modifier.fillMaxWidth()
                )
                OutlinedTextField(
                    value = reviewInterval,
                    onValueChange = { reviewInterval = it },
                    label = { Text("Интервал проверки, с") },
                    isError = !reviewValid,
                    supportingText = { if (!reviewValid) Text("Введите целое число допустимого размера") },
                    placeholder = { Text("Число секунд") },
                    singleLine = true,
                    enabled = !isLoading,
                    modifier = Modifier.fillMaxWidth()
                )
            }
        },
        confirmButton = {
            Button(
                onClick = { onConfirm(expiredNotify, reviewInterval) },
                enabled = !isLoading && expiredValid && reviewValid
            ) {
                if (isLoading) {
                    CircularProgressIndicator(
                        modifier = Modifier.size(18.dp),
                        color = MaterialTheme.colorScheme.onPrimary,
                        strokeWidth = 2.dp
                    )
                } else {
                    Text("Сохранить")
                }
            }
        },
        dismissButton = {
            TextButton(onClick = onDismiss, enabled = !isLoading) {
                Text("Отмена")
            }
        }
    )
}

@Composable
fun InviteTokenDialog(
    token: String,
    onDismiss: () -> Unit
) {
    val clipboardManager = LocalClipboardManager.current
    val context = LocalContext.current

    PreviewableAlertDialog(
        onDismissRequest = onDismiss,
        title = { Text("Приглашение в комнату") },
        text = {
            Column(
                verticalArrangement = Arrangement.spacedBy(12.dp),
                horizontalAlignment = Alignment.CenterHorizontally
            ) {
                Text(
                    text = "Передайте этот токен новому участнику для входа в комнату:",
                    style = MaterialTheme.typography.bodyMedium
                )
                Surface(
                    shape = RoundedCornerShape(8.dp),
                    color = MaterialTheme.colorScheme.surfaceContainerHigh,
                    modifier = Modifier.fillMaxWidth()
                ) {
                    Text(
                        text = token,
                        style = MaterialTheme.typography.titleMedium,
                        fontFamily = FontFamily.Monospace,
                        fontWeight = FontWeight.Bold,
                        color = MaterialTheme.colorScheme.primary,
                        textAlign = TextAlign.Center,
                        modifier = Modifier.padding(16.dp)
                    )
                }
            }
        },
        confirmButton = {
            Button(
                onClick = {
                    clipboardManager.setText(AnnotatedString(token))
                    Toast.makeText(context, "Токен скопирован в буфер обмена", Toast.LENGTH_SHORT)
                        .show()
                    onDismiss()
                }
            ) {
                Icon(
                    Icons.Default.Share,
                    contentDescription = null,
                    modifier = Modifier.size(18.dp)
                )
                Spacer(modifier = Modifier.width(8.dp))
                Text("Скопировать")
            }
        },
        dismissButton = {
            TextButton(onClick = onDismiss) {
                Text("Закрыть")
            }
        }
    )
}

@Composable
fun DeleteRoomDialog(
    isLoading: Boolean,
    onDismiss: () -> Unit,
    onConfirm: () -> Unit
) {
    PreviewableAlertDialog(
        onDismissRequest = onDismiss,
        title = { Text("Удалить комнату?") },
        text = { Text("Вы уверены, что хотите удалить эту комнату? Это действие нельзя отменить.") },
        confirmButton = {
            Button(
                onClick = onConfirm,
                enabled = !isLoading,
                colors = ButtonDefaults.buttonColors(containerColor = MaterialTheme.colorScheme.error)
            ) {
                if (isLoading) {
                    CircularProgressIndicator(
                        modifier = Modifier.size(18.dp),
                        color = MaterialTheme.colorScheme.onError,
                        strokeWidth = 2.dp
                    )
                } else {
                    Text("Удалить")
                }
            }
        },
        dismissButton = {
            TextButton(onClick = onDismiss, enabled = !isLoading) {
                Text("Отмена")
            }
        }
    )
}

// --- Previews ---

private object PreviewSample {

    val room = Room(
        id = "room-1234-5678-abcd-000000000001",
        displayName = "Кладовка #1",
        bio = "Главный склад запасов и инструментов на даче.",
        createdAt = "2026-01-15T10:30:00Z",
        expiredNotify = 86400L,
        reviewInterval = 2592000L,
    )

    val roomNoBio = room.copy(bio = "")
    val roomNoTimings = room.copy(expiredNotify = null, reviewInterval = null)

    val roomLong = room.copy(
        displayName = "Очень длинное название комнаты для кладовки с инструментами и запасами",
        bio = buildString {
            repeat(6) {
                append("Это длинное описание, которое проверяет перенос строк и обрезку. ")
            }
        },
    )

    fun member(
        id: String,
        displayName: String,
        bio: String,
        userId: String? = null,
    ) = Member(
        id = id,
        roomId = room.id,
        userId = userId,
        displayName = displayName,
        bio = bio,
        createdAt = "2026-01-15T10:30:00Z",
    )

    val members = listOf(
        member("m1", "Анна", "Администратор", "u1"),
        member("m2", "Иван", "Участник", "u2"),
        member("m3", "Ольга", "", "u3"),
        member("m4", "Гость", "Приглашён по ссылке"), // без аккаунта, userId = null
    )

    val manyMembers: List<Member> = (1..30).map { i ->
        member(
            id = "m$i",
            displayName = "Участник $i",
            bio = if (i % 2 == 0) "Роль $i" else "",
            userId = if (i % 3 == 0) null else "u$i",
        )
    }
}

private class PreviewMembers(
    private val items: List<Member>,
    override val isRefreshing: Boolean = false,
    override val isLoadingMore: Boolean = false,
) : PagingSourceList<Member> {
    override val itemCount: Int get() = items.size
    override fun get(index: Int): Member? = items.getOrNull(index)
    override val refreshError: Throwable? = null
    override val appendError: Throwable? = null
    override fun retry() = Unit
}

// ---------- Экран: состояния ----------

@Preview(showBackground = true, name = "01 — Загрузка")
@Composable
private fun RoomProfileLoadingPreview() {
    ZapasTheme {
        RoomProfileScreenContent(
            members = PreviewMembers(emptyList(), isRefreshing = true),
            state = RoomProfileState(isLoading = true, room = null),
            onBack = {}, onRefresh = {},
            onShowEditDialog = {}, onShowTimingsDialog = {},
            onGenerateInvite = {}, onShowDeleteDialog = {},
        )
    }
}

@Preview(showBackground = true, name = "02 — Ошибка загрузки")
@Composable
private fun RoomProfileErrorPreview() {
    ZapasTheme {
        RoomProfileScreenContent(
            members = PreviewMembers(emptyList()),
            state = RoomProfileState(
                room = null,
                isLoading = false,
                error = "Не удалось загрузить комнату. Проверьте подключение к сети.",
            ),
            onBack = {}, onRefresh = {},
            onShowEditDialog = {}, onShowTimingsDialog = {},
            onGenerateInvite = {}, onShowDeleteDialog = {},
        )
    }
}

@Preview(showBackground = true, name = "03 — Успех, с участниками")
@Composable
private fun RoomProfileSuccessPreview() {
    ZapasTheme {
        RoomProfileScreenContent(
            members = PreviewMembers(PreviewSample.members),
            state = RoomProfileState(room = PreviewSample.room),
            onBack = {}, onRefresh = {},
            onShowEditDialog = {}, onShowTimingsDialog = {},
            onGenerateInvite = {}, onShowDeleteDialog = {},
        )
    }
}

@Preview(showBackground = true, name = "04 — Без участников")
@Composable
private fun RoomProfileNoMembersPreview() {
    ZapasTheme {
        RoomProfileScreenContent(
            members = PreviewMembers(emptyList()),
            state = RoomProfileState(room = PreviewSample.room),
            onBack = {}, onRefresh = {},
            onShowEditDialog = {}, onShowTimingsDialog = {},
            onGenerateInvite = {}, onShowDeleteDialog = {},
        )
    }
}

@Preview(showBackground = true, name = "05 — Без описания")
@Composable
private fun RoomProfileNoBioPreview() {
    ZapasTheme {
        RoomProfileScreenContent(
            members = PreviewMembers(PreviewSample.members.take(2)),
            state = RoomProfileState(room = PreviewSample.roomNoBio),
            onBack = {}, onRefresh = {},
            onShowEditDialog = {}, onShowTimingsDialog = {},
            onGenerateInvite = {}, onShowDeleteDialog = {},
        )
    }
}

@Preview(showBackground = true, name = "06 — Без таймингов")
@Composable
private fun RoomProfileNoTimingsPreview() {
    ZapasTheme {
        RoomProfileScreenContent(
            members = PreviewMembers(PreviewSample.members.take(1)),
            state = RoomProfileState(room = PreviewSample.roomNoTimings),
            onBack = {}, onRefresh = {},
            onShowEditDialog = {}, onShowTimingsDialog = {},
            onGenerateInvite = {}, onShowDeleteDialog = {},
        )
    }
}

@Preview(showBackground = true, name = "07 — Длинный контент")
@Composable
private fun RoomProfileLongContentPreview() {
    ZapasTheme {
        RoomProfileScreenContent(
            members = PreviewMembers(PreviewSample.manyMembers),
            state = RoomProfileState(room = PreviewSample.roomLong),
            onBack = {}, onRefresh = {},
            onShowEditDialog = {}, onShowTimingsDialog = {},
            onGenerateInvite = {}, onShowDeleteDialog = {},
        )
    }
}

@Preview(
    showBackground = true,
    name = "08 — Тёмная тема",
    uiMode = Configuration.UI_MODE_NIGHT_YES,
)
@Composable
private fun RoomProfileDarkPreview() {
    ZapasTheme {
        RoomProfileScreenContent(
            members = PreviewMembers(PreviewSample.members),
            state = RoomProfileState(room = PreviewSample.room),
            onBack = {}, onRefresh = {},
            onShowEditDialog = {}, onShowTimingsDialog = {},
            onGenerateInvite = {}, onShowDeleteDialog = {},
        )
    }
}

// ---------- Диалоги ----------

@Preview(showBackground = true, name = "Диалог — редактирование")
@Composable
private fun EditRoomDialogPreview() {
    ZapasTheme {
        EditRoomProfileDialog(
            currentName = PreviewSample.room.displayName,
            currentBio = PreviewSample.room.bio,
            isLoading = false,
            onDismiss = {},
            onConfirm = { _, _ -> },
        )
    }
}

@Preview(showBackground = true, name = "Диалог — редактирование (загрузка)")
@Composable
private fun EditRoomDialogLoadingPreview() {
    ZapasTheme {
        EditRoomProfileDialog(
            currentName = PreviewSample.room.displayName,
            currentBio = PreviewSample.room.bio,
            isLoading = true,
            onDismiss = {},
            onConfirm = { _, _ -> },
        )
    }
}

@Preview(showBackground = true, name = "Диалог — тайминги")
@Composable
private fun UpdateTimingsDialogPreview() {
    ZapasTheme {
        UpdateTimingsDialog(
            currentExpiredNotify = PreviewSample.room.expiredNotify?.toString().orEmpty(),
            currentReviewInterval = PreviewSample.room.reviewInterval?.toString().orEmpty(),
            isLoading = false,
            onDismiss = {},
            onConfirm = { _, _ -> },
        )
    }
}

@Preview(showBackground = true, name = "Диалог — тайминги (загрузка)")
@Composable
private fun UpdateTimingsDialogLoadingPreview() {
    ZapasTheme {
        UpdateTimingsDialog(
            currentExpiredNotify = PreviewSample.room.expiredNotify?.toString().orEmpty(),
            currentReviewInterval = PreviewSample.room.reviewInterval?.toString().orEmpty(),
            isLoading = true,
            onDismiss = {},
            onConfirm = { _, _ -> },
        )
    }
}

@Preview(showBackground = true, name = "Диалог — инвайт")
@Composable
private fun InviteTokenDialogPreview() {
    ZapasTheme {
        InviteTokenDialog(
            token = "eyJhbGciOiJIUzI1NiJ9.eyJyb29tX2lkIjoicm9vbS0xMjM0In0.demo-signature",
            onDismiss = {},
        )
    }
}

@Preview(showBackground = true, name = "Диалог — удаление")
@Composable
private fun DeleteRoomDialogPreview() {
    ZapasTheme {
        DeleteRoomDialog(
            isLoading = false,
            onDismiss = {},
            onConfirm = {},
        )
    }
}

@Preview(showBackground = true, name = "Диалог — удаление (загрузка)")
@Composable
private fun DeleteRoomDialogLoadingPreview() {
    ZapasTheme {
        DeleteRoomDialog(
            isLoading = true,
            onDismiss = {},
            onConfirm = {},
        )
    }
}