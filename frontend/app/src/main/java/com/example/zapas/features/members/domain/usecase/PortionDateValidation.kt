package com.example.zapas.features.members.domain.usecase

import java.text.ParsePosition
import java.text.SimpleDateFormat
import java.util.Locale
import java.util.TimeZone

// String dates avoid requiring java.time desugaring on Android 7 (minSdk 24).
internal fun isValidPortionDate(date: String): Boolean {
    if (!Regex("[0-9]{4}-[0-9]{2}-[0-9]{2}").matches(date)) return false
    val format = SimpleDateFormat("yyyy-MM-dd", Locale.ROOT).apply {
        isLenient = false
        timeZone = TimeZone.getTimeZone("UTC")
        gregorianChangeCompat()
    }
    val position = ParsePosition(0)
    return format.parse(date, position) != null && position.index == date.length
}

private fun SimpleDateFormat.gregorianChangeCompat() {
    (calendar as? java.util.GregorianCalendar)?.gregorianChange = java.util.Date(Long.MIN_VALUE)
}
