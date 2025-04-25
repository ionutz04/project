library(stringr)
lines <- readLines("data.txt")
amp <- numeric()
freq <- numeric()
for (line in lines) {
    if (grepl("Freq:", line) & grepl("Amp:", line)) {
        freq_match <- str_extract(line, "Freq: (\\d+\\.\\d+) Hz")
        amp_match <- str_extract(line, "Amp: (\\d+\\.\\d+) V")
        
        # Extract the numeric values
        if (!is.na(freq_match) & !is.na(amp_match)) {
        freq_value <- as.numeric(gsub("Freq: | Hz", "", freq_match))
        amp_value <- as.numeric(gsub("Amp: | V", "", amp_match))
        
        freq <- c(freq, freq_value)
        amp <- c(amp, amp_value)
        }
    }
}
plot(x = freq, # x-coordinates
     y = amp, # y-coordinates
     type = "p", # Plot points (not lines)
     main = "Frequency vs Amplitude", # Title of the plot
     xlab = "Frequency (Hz)", # Label for x-axis
     ylab = "Amplitude (V)", # Label for y-axis
     xlim = c(min(freq) - 0.1, max(freq) + 0.1), # Set x-axis limits
     ylim = c(min(amp) - 0.01, max(amp) + 0.01), # Set y-axis limits
     col = "blue", # Color of the points
     pch = 19, # Type of symbol (filled circle)
     cex = 2) # Size of the symbol
