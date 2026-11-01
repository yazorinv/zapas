package com.example.zapas.features.members.presentation.ui

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.Refresh
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.ListItem
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedCard
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.example.zapas.features.members.domain.model.MemberPortion
import com.example.zapas.features.members.presentation.viewmodel.MemberProfileState
import com.example.zapas.features.members.presentation.viewmodel.MemberProfileViewModel
import com.example.zapas.features.rooms.domain.model.Member
import com.example.zapas.ui.theme.ZapasTheme

@Composable
fun MemberProfileScreen(
    onBack: () -> Unit,
    viewModel: MemberProfileViewModel = hiltViewModel()
) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    MemberProfileScreenContent(state, onBack, viewModel::load)
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun MemberProfileScreenContent(
    state: MemberProfileState,
    onBack: () -> Unit,
    onRefresh: () -> Unit
) {
    Scaffold(topBar = {
        TopAppBar(
            title = { Text("Участник комнаты") },
            navigationIcon = {
                IconButton(onClick = onBack) {
                    Icon(Icons.AutoMirrored.Filled.ArrowBack, contentDescription = "Назад")
                }
            },
            actions = {
                IconButton(onClick = onRefresh) {
                    Icon(Icons.Default.Refresh, contentDescription = "Обновить")
                }
            }
        )
    }) { padding ->
        LazyColumn(
            modifier = Modifier.fillMaxSize().padding(padding),
            contentPadding = PaddingValues(16.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            item {
                if (state.isLoading) CircularProgressIndicator()
                state.error?.let { MemberError(it, onRefresh) }
            }
            state.member?.let { member ->
                item {
                    OutlinedCard(modifier = Modifier.fillMaxWidth()) {
                        Column(
                            modifier = Modifier.padding(16.dp),
                            verticalArrangement = Arrangement.spacedBy(8.dp)
                        ) {
                            Text(member.displayName.ifBlank { "Без имени" }, style = MaterialTheme.typography.headlineSmall)
                            if (state.isOwnProfile == true)
                                Text("Это вы", color = MaterialTheme.colorScheme.primary)
                            if (member.bio.isNotBlank()) Text(member.bio)
                        }
                    }
                }
                state.identityError?.let { error ->
                    item { MemberError("Не удалось определить текущего пользователя: $error", onRefresh) }
                }
                item {
                    OutlinedCard(modifier = Modifier.fillMaxWidth()) {
                        Column(
                            modifier = Modifier.padding(16.dp),
                            verticalArrangement = Arrangement.spacedBy(8.dp)
                        ) {
                            Text("Порций по умолчанию", style = MaterialTheme.typography.titleMedium)
                            if (state.isDefaultPortionsLoading) CircularProgressIndicator(modifier = Modifier.size(24.dp))
                            state.defaultPortions?.let { Text(it.toString()) }
                            state.defaultPortionsError?.let { MemberError(it, onRefresh) }
                        }
                    }
                }
                item {
                    Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                        Text("Порции за ${state.periodTitle}", style = MaterialTheme.typography.titleMedium)
                        if (state.isPortionsLoading) CircularProgressIndicator(modifier = Modifier.size(24.dp))
                        state.portionsError?.let { MemberError(it, onRefresh) }
                        if (!state.isPortionsLoading && state.portionsError == null && state.portions.isEmpty())
                            Text("За этот период записей нет")
                    }
                }
                items(state.portions) { record ->
                    Column {
                        ListItem(
                            headlineContent = { Text(record.date) },
                            trailingContent = { Text(record.portions.toString()) }
                        )
                        HorizontalDivider()
                    }
                }
            }
        }
    }
}

@Composable
private fun MemberError(error: String, onRetry: () -> Unit) {
    Column {
        Text(error, color = MaterialTheme.colorScheme.error)
        TextButton(onClick = onRetry) { Text("Повторить") }
    }
}

@Preview(showBackground = true)
@Composable
private fun MemberProfilePreview() {
    ZapasTheme {
        MemberProfileScreenContent(
            state = MemberProfileState(
                member = Member("Описание в комнате", "2026-11-01T00:00:00Z", "Алексей", "member", "room", "user"),
                currentUserId = "user",
                defaultPortions = 1,
                portions = listOf(MemberPortion("2026-11-01", 0), MemberPortion("2026-11-02", 2)),
                periodTitle = "ноябрь 2026",
                isLoading = false,
                isDefaultPortionsLoading = false,
                isPortionsLoading = false
            ),
            onBack = {},
            onRefresh = {}
        )
    }
}
